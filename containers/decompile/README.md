# Decompile Minecraft Binaries

This container uses [`DecompilerMC`](https://github.com/hube12/DecompilerMC) to
decompile minecraft binaries for versions 1.14.4 or later.

## Usage

Use the script:

```
$ ./decompile-client.sh
```

The following environment variables can be set to specify options to the script:

* `VERSION`: Minecraft version of the binary to decompile.
  * Default: `"1.14.4"`

* `DESTDIR`: Directory on the host machine to save the decompiled source code
  files to.
  * Default: `${BRINE_ROOT}/containers/decompile/src/`

* `DOCKER_IMAGE_NAME`: The name to tag the built docker image with.
  * Default: `mc-decompile`

Example:

```
$ VERSION="1.18" DESTDIR="~/.minecraft/versions/1.18/src/" ./decompile-client.sh
```