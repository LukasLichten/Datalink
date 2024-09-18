# Datalink
Simple wrapper around Steam/Proton Games that sends notification of their launch and deploys a Shared memory bridge

## Usage
Apppend Datalink infront of the launch command (same way as Mangohud) by setting the Launch Options on Steam like this:
```
Datalink %command%
```

You are free to combine this with Mangohud and or regular launch options

## Programmatical Usage
For writing game tools this wrapper exposes resources (memory maps) and notifies when the game is launched (so you can start reading data).  
  
*TODO not implemented*

### Shared Memory And The Bridge
### Configuring Memory Maps
### Game Status Notification


## Building
Requires rust, but also gnu-windows cross compile target:
```
rustup target add x86_64-pc-windows-gnu
```
  
Project can be build via
```
make
```
  
Your lsp might flag the `include_bytes` line, use
```
make debug-build
```
to build the missing bridge.exe in debug, which will fix the missing file.  
  
For debugging you can add the full path to the binary (`[...]/Datalink/target/release/Datalink`)
to the Launch Option of the game you use for testing.  
To get debug output you can cold start steam from the terminal, then the first stage will log into that terminal session
(however the second stage will not, but is currently erroneously spawning a wine-terminal window).

## Related

This project (in large parts) is a fork of [shm-bridge](https://github.com/poljar/shm-bridge).  
  
Other related Projects can be found here: [awesome-linux-simeracing/libraries and dev utils](https://github.com/LukasLichten/awesome-linux-simracing#libraries-headers-and-other-dev-utils)

