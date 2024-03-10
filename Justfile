set dotenv-load := true

toolchain:
    #!/usr/bin/env bash
    mkdir -p toolchain
    if (rustup toolchain list | grep -q "esp"); then
        espup update \
            --log-level info \
            --targets esp32 \
            --export-file export-esp-rust.sh \
            --extended-llvm
    else
        espup install \
            --log-level info \
            --targets esp32 \
            --export-file export-esp-rust.sh \
            --extended-llvm
    fi

clean:
    espup uninstall

flash *args:
    #!/usr/bin/env bash
    set -euxo pipefail
    set -o allexport; source export-esp-rust.sh; set +o allexport
    cargo espflash flash --monitor {{ args }}
