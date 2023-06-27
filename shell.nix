{ pkgs ? import <nixpkgs> { } }:
pkgs.mkShell rec {
  buildInputs = with pkgs; [
    clang
    openssl
    pkg-config
    llvmPackages.bintools
    rustup
    cargo-cross
    alsa-lib
    exa
    lldb
    glibc.dev
  ];
  RUSTC_VERSION = pkgs.lib.readFile ./rust-toolchain;
  # https://github.com/rust-lang/rust-bindgen#environment-variables
  LIBCLANG_PATH =
    pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];
  RUST_BACKTRACE = 1;

  LD_LIBRARY_PATH = with pkgs;
    lib.makeLibraryPath [
      libGL
      libxkbcommon
      wayland
      xorg.libX11
      xorg.libXcursor
      xorg.libXi
      xorg.libXrandr
      alsa-lib
      # vulkan-loader
    ];

  NIX_LD = pkgs.lib.fileContents "${pkgs.stdenv.cc}/nix-support/dynamic-linker";

  shellHook = ''
    export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
    export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/
    alias ls=exa
  '';
  # Add precompiled library to rustc search path
  RUSTFLAGS = (builtins.map (a: "-L ${a}/lib") [
    # add libraries here (e.g. pkgs.libvmi)
  ]);
  # Add glibc, clang, glib and other headers to bindgen search path
  BINDGEN_EXTRA_CLANG_ARGS =
    # Includes with normal include path
    (builtins.map (a: ''-I"${a}/include"'') [
      # add dev libraries here (e.g. pkgs.libvmi.dev)
      pkgs.glibc.dev
    ])
    # Includes with special directory paths
    ++ [
      ''
        -I"${pkgs.llvmPackages_latest.libclang.lib}/lib/clang/${pkgs.llvmPackages_latest.libclang.version}/include"''
      ''-I"${pkgs.glib.dev}/include/glib-2.0"''
      "-I${pkgs.glib.out}/lib/glib-2.0/include/"
    ];

}
