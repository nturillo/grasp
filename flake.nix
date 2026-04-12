{
  description = "Grasp Development";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.11";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix/monthly";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs: with inputs;
    flake-utils.lib.eachDefaultSystem (system: let
      rt = fenix.packages.${system}.latest.withComponents [
        "cargo"
        "clippy"
        "rust-src"
        "rustc"
        "rustfmt"
        "rust-analyzer"
      ];
      pkgs = import nixpkgs {inherit system;};
    in {
      formatter = pkgs.nixpkgs-fmt;
      inherit rt;
      devShells.default = pkgs.mkShell rec {
        name = "Grasp Development";

        nativeBuildInputs = with pkgs; [
          pkg-config
          cargo-udeps
          rt
          git
          mold
          python312
        ];
        buildInputs = with pkgs; [
          libxcb libxkbcommon openssl 
          libGL vulkan-loader
          libx11 libxi libxcursor libxrandr # To use the x11 feature
          sqlite
          rt
          wayland
          python312
        ];

        # needed for rust-analyzer
        RUST_SRC_PATH = "${rt}/lib/rustlib/src/rust/library";
        LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
      };
      }
    );
}
