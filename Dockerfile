FROM notfl3/cargo-apk

RUN apt update && apt install -y pkg-config libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev mingw-w64 && rm -rf /var/lib/apt/lists/*
RUN cargo install cross
RUN rustup target add x86_64-pc-windows-gnu
RUN rustup target add x86_64-unknown-linux-gnu
RUN rustup target add i686-pc-windows-gnu

RUN mkdir -p ~/.cargo/ && \
    echo "[target.x86_64-pc-windows-gnu]" > ~/.cargo/config.toml && \
    echo "linker = 'x86_64-w64-mingw32-gcc'" >> ~/.cargo/config.toml && \
    echo "ar = 'x86_64-w64-mingw32-gcc-ar'" >> ~/.cargo/config.toml

RUN mkdir -p /root/target_link/ehce_cache
RUN ln -s /root/target_link/ehce_cache /root/src/target

COPY . /root/src

CMD \
    cargo build --target x86_64-pc-windows-gnu"$buildArgs" -p game | tail -n +5 | sed -e '1,4d' && echo "x86_64-pc-windows-gnu Done!" && \
    cargo build --target x86_64-unknown-linux-gnu"$buildArgs" -p game | tail -n +5 | sed -e '1,4d' && echo "x86_64-unknown-linux-gnu Done!" && \
    cargo quad-apk build"$buildArgs" -p game | tail -n +5 | sed -e '1,4d' && echo "x86_64-pc-windows-gnu Done!"