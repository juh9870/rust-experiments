use std::fs;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use anyhow::{anyhow, bail};
use lazy_static::lazy_static;
use poise::serenity_prelude::GatewayIntents;
use poise::PrefixFrameworkOptions;
use rand::Rng;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::Deserialize;
use tokio::time::sleep;

struct Data {
    waifus: WaifuData,
    mappings: FxHashMap<String, usize>,
}

// User data, which is stored and accessible in all command invocations
type Error = anyhow::Error;
type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Debug, Deserialize)]
struct RawWaifuCategory {
    weight: f64,
    items: Vec<String>,
}

#[derive(Debug)]
struct WaifuCategory {
    name: String,
    items: Vec<String>,
}

#[derive(Debug)]
struct WaifuData {
    weights: Vec<f64>,
    categories: Vec<WaifuCategory>,
}

#[derive(Debug, Clone)]
struct Inventory {
    content: FxHashMap<usize, u32>,
}

lazy_static! {
    static ref DATA_DIR: PathBuf =
        Path::new(&std::env::var("WAIFU_DIR").unwrap_or_else(|_| ".".to_owned())).to_path_buf();
}

fn load_waifus() -> Result<WaifuData, Error> {
    let Ok(data) = fs::read_to_string(DATA_DIR.join(Path::new("waifus.json"))) else {
        bail!("Failed to read waifus.json")
    };
    let Ok(data) = serde_json::from_str::<FxHashMap<String, RawWaifuCategory>>(data.trim_start_matches("\u{feff}")) else {
        bail!("Invalid waifu file")
    };
    let (weights, categories) = data
        .into_iter()
        .map(|(k, v)| {
            (
                v.weight,
                WaifuCategory {
                    name: k,
                    items: v.items,
                },
            )
        })
        .unzip();

    Ok(WaifuData {
        weights,
        categories,
    })
}

fn update_mappings(waifus: &WaifuData) -> Result<FxHashMap<String, usize>, Error> {
    let mappings_path = DATA_DIR.join(Path::new("mappings.json"));
    let data = fs::read_to_string(mappings_path.clone()).unwrap_or("[]".to_owned());

    let Ok(mut mappings) =
        serde_json::from_str::<Vec<String>>(data.trim_start_matches("\u{feff}")) else {
        bail!("Failed to deserialize mappings")
    };

    let mut to_add = Vec::new();
    {
        let mut mappings_map = mappings
            .iter()
            .enumerate()
            .map(|(i, e)| (e, i))
            .collect::<FxHashMap<_, _>>();
        let mut waifu_set = FxHashSet::default();
        for e in waifus.categories.iter().flat_map(|e| e.items.iter()) {
            if e.starts_with("//") {
                bail!("Waifu names can't start with double forward slash. At: \"{e}\"");
            }
            if !waifu_set.insert(e) {
                bail!("Duplicate item found: {e}");
            }
            if mappings_map.remove(e).is_none() {
                to_add.push(e.clone());
            }
        }
        let extra_keys = mappings_map
            .into_keys()
            .filter(|e| !e.starts_with("//"))
            .collect::<Vec<_>>();
        if !extra_keys.is_empty() {
            let extra = extra_keys
                .into_iter()
                .map(|e| format!("\"{e}\""))
                .collect::<Vec<_>>()
                .join(",");
            bail!(
                "Some keys were present in mappings but missing in waifu list:\n{extra}\n\
                If entries were renamed then rename mapping entries accordingly.\n\
                If those entries got permanently deleted, then edit these entries in `mappings.json` \
                to start with two forward slashes.\nFor example, \"Some Old Entry\" -> \"//Reason for deletion\""
            );
        }
    };
    mappings.append(&mut to_add);

    let Ok(mappings_json) = serde_json::to_string_pretty(&mappings) else { bail!("Failed to serialize mappings") };
    if fs::write(mappings_path, mappings_json).is_err() {
        bail!("Failed to write mappings file")
    }
    println!("Mapping reloaded successfully");
    Ok(mappings
        .into_iter()
        .enumerate()
        .map(|(i, e)| (e, i))
        .collect())
}

fn random_weighted<R: Rng>(weights: &Vec<f64>, rng: &mut R) -> Option<usize> {
    let total: f64 = weights.iter().sum();
    let mut roll = rng.gen_range(0.0..total);

    for (i, x) in weights.iter().enumerate() {
        roll -= x;
        if roll <= 0.0 {
            return Some(i);
        }
    }
    None
}

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command)]
async fn roll(
    ctx: Context<'_>,
    #[description = "Amount of rolls"] rolls: Option<u32>,
) -> Result<(), Error> {
    let mut response: Vec<String> = Vec::new();
    let rolls = rolls.unwrap_or(1);
    if rolls > 10 {
        bail!("Too many rolls")
    }
    let msg = ctx.say("Rolling...").await?;
    for i in 1..=rolls {
        response.push({
            let mut rand = rand::thread_rng();
            let category_index = random_weighted(&ctx.data().waifus.weights, &mut rand)
                .ok_or(anyhow!("Roll failed, please contact bot author"))?;
            let category = &ctx.data().waifus.categories[category_index];

            let roll = rand.gen_range(0..category.items.len());

            let waifu = &category.items[roll];

            // let u = user.as_ref().unwrap_or_else(|| ctx.author());
            format!("{i}: You have pulled «{}» **{}**", category.name, waifu)
        });
        sleep(Duration::from_secs(1)).await;
        msg.edit(ctx, |e| {
            e.content = Some(response.join("\n"));
            e
        })
        .await?;
    }

    Ok(())
}

#[poise::command(prefix_command)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    // dbg!(&WAIFUS
    //     .categories
    //     .iter()
    //     .map(|e| &e.name)
    //     .collect::<Vec<_>>());
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![register(), roll()],
            prefix_options: PrefixFrameworkOptions {
                prefix: Some("rw!".to_string()),
                ..Default::default()
            },
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT)
        .setup(|ctx, _ready, _framework| {
            Box::pin(async move {
                println!("Loading waifus...");
                let waifus = load_waifus().unwrap();
                println!("Loading mapping...");
                let mappings = update_mappings(&waifus).unwrap();
                println!("Done!");
                // poise::builtins::register_in_guild(
                //     ctx,
                //     &framework.options().commands,
                //     GuildId(216795736602443777),
                // )
                // .await?;
                // poise::builtins::register_application_commands_buttons(ctx).await?;
                // poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data { waifus, mappings })
            })
        });

    framework.run().await.unwrap();
}
