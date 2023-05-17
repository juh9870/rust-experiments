if [ -z "$1" ]
then
    echo "No argument passed. Defaulting to 'debug'."
    mode="debug"
elif [ "$1" != "release" ] && [ "$1" != "debug" ]
then
    echo "Invalid argument: $1. Must be 'release' or 'debug'."
    exit 1
else
    mode="$1"
fi

echo "Running in $mode mode"

if [ "$mode" == "release" ]
then
    buildArgs=" --release"
else
    buildArgs=""
fi

# exit 0
mkdir -p ./build/release
mkdir -p ./build/debug
docker build . -t macroquad_test
docker image prune -f
docker run --env buildArgs="$buildArgs" -v cargo_cache:/usr/local/cargo/registry/ -v build_cache:/root/target_link/ --name macroquad_test_builds macroquad_test
# docker compose up --build
docker cp macroquad_test_builds:/root/src/target/android-artifacts/"$mode"/apk/game.apk ./build/"$mode"/android.apk
docker cp macroquad_test_builds:/root/src/target/x86_64-pc-windows-gnu/"$mode"/game.exe ./build/"$mode"/win64.exe
docker cp macroquad_test_builds:/root/src/target/x86_64-unknown-linux-gnu/"$mode"/game ./build/"$mode"/linux64
docker rm macroquad_test_builds
# docker compose down