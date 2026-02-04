Mostly raw bindings to the 1Password SDK.

## Setup
The dynamic libraries required can be downloaded from https://github.com/1Password/onepassword-sdk-python/tree/main/src/onepassword/lib

## Notes
`pollster` is included because getting a client ID requires polling a future no matter what, but since it's our own future we know `pollster` works fine.