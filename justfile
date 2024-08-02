_default:
    @just --list

@dev:
    cargo build -q
    ROFI_PLUGIN_PATH=target/debug rofi -modi faye -show faye

@build:
    cargo build --release

@install: (build)
    sudo cp -u ./target/release/libfaye.so /lib/rofi/faye.so
