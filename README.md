# clc in rust
    Opting for the most manual experience, this is truly the best
    and securestest chat solution out there
    - someone
_[for legal reasons, the above is a joke]_

<br>

The cli was developed on a Windows machine, so the cli might not work as expected 
on other platforms. If so, please open an [issue](https://github.com/DragonFIghter603/command-line-chat/issues),
I will try to get it working then.

IDE terminals, such as the ones of jetbrains oftentimes do not play well with a cli.
I suggest using the default windows cmd.

Info: Debug logs are omitted from release build

`/?` list of [commands](clc-client/command-help.md) 

## Contents
- [clc-client](clc-client) the client side cli application
- [clc-lib](clc-lib) protocol and information transfer
- [clc-server](clc-server) the server side application

## How to use
1. clone or download this repo
2. open cmd, cd to `clc-client`
3. run `cargo build --release`
4. (optional) add the location of the exe to `PATH`
5. run `clc-client` from cmd (either in `build/release` dir or froom anywhere when exe is in `PATH`)

Alternatively, download the latest build from the [release page](https://github.com/DragonFIghter603/command-line-chat/releases)

## Terminology
- cli `command line interface`
- clc `command line chat`
- clcs `command line chat server`
- clcc `command line chat client`
- clcccli `command line chat client command line interface`