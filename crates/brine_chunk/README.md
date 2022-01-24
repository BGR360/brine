# brine_chunk

This library does two things:

1. Provides data type definitions for Minecraft chunk data, used by
   `brine_proto`.

2. Provides methods to decode chunk data from its various[^1] different
   compressed formats.

[^1]: Eventually. Currently only supports decoding 1.14.4 chunk data.