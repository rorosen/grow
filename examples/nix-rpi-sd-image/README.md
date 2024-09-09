# Nix SD Image

This example shows how to build a pre-configured image for a Raspberry Pi 4 that you can flash to an
SD card and run right away.

You may want to place your SSH key in the [configuration](./configuration.nix) to get access to the
system. If you don't have Nix installed already, check out the
[Determinate Nix Installer](https://determinate.systems/posts/determinate-nix-installer/).

```shell
# Build the image
nix build .\#packages.aarch64-linux.sdImage
# Flash it to an SD card, replace sdX with the right value (check with lsblk)
sudo dd if=result/sd-image/nixos-sd-image-*-aarch64-linux.img of=/dev/sdX bs=4M
```

After booting the Pi, you can make changes to the configuration and remotely rebuild the system.

```shell
# Rebuild the configuration, replace the IP address to match that of your Pi
nixos-rebuild switch --flake .\#pi --target-host root@192.168.123.123
```

The configuration uses the `-aarch64` packages in order to enable cross-compiling from `x86_64`,
which is way faster than emulating `aarch64-linux` for the Rust build. However, the rest of the
build is emulated so we can use the Nix binary cache. If you are on `x86_64` and want to emulate an
`aarch64-linux` system, you need to enable [binfmt](https://en.wikipedia.org/wiki/Binfmt_misc) for
that system. In NixOS this can be done via the `boot.binfmt.emulatedSystems` option.

```nix
boot.binfmt.emulatedSystems = [ "aarch64-linux" ];
```
