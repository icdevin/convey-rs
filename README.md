# Convey-rs

A Rust library for working with Satisfactory save files. Most of the credit
goes to the [Satisfactory Calculator Interactive Map][0] and the
[Satisfactory Wiki][1].

Any and all assets, tradements, and registered trademarks belonging to the
Satisfactory game are the exclusive property of Coffee Stain Studios. This
project is in no way affiliated with or endorsed by Coffee Stain Studios.

## The Save File

The Satisfactory `.sav` file comes from [Unreal Engine][2]*, which provides a
default implementation for saving and loading games provided game developers
provide a `SaveGame` class/classes. Because game developers can provide almost
any arbitrary variables to be stored in the save file, the parsed format likely
widely varies from game-to-game. However, provided the default save game
implementation is used, the underlying algorithm for serializing the chosen
data should be the same.

The default save file format is reminiscent of many others, where an unencoded
header at the beginning contains some meta information about the data stored in
the body. The entire save file is not blanket-encoded in some other format
(e.g. JSON) but rather as plain bytes.

\* Satisfactory is [currently built using UE 5][3].

### Header

The header contains metadata about the save itself as well as the game, probably
to do validation and display saves on the game UI without needing to load the
entire file's contents.

The [docs][4] contain the header fields in the order which they appear (i.e. the
`save_header_version` is the first 4 bytes, the `save_file_version` is the next
4 bytes, and so on).

### Body

The body begins the very next byte after the last header field. The body is
split into chunks and each chunk itself has a small header and a zlib-compressed
payload. This library does not retain or return information about these chunks
as their purpose is merely to reduce the overall file size. There is no padding
between chunks and their contents should be decompressed first and then
concatenated to produce the whole body. There is no more data after the body,
i.e. the last chunk's payload should be the EOF.

[0]: https://satisfactory-calculator.com/en/interactive-map "Satisfactory Calculator Interactive Map"
[1]: https://satisfactory.wiki.gg/wiki/Save_files "Satisfactory Wiki save file page"
[2]: https://dev.epicgames.com/documentation/en-us/unreal-engine/saving-and-loading-your-game-in-unreal-engine "Saving/loading games in Unreal Engine"
[3]: https://www.youtube.com/watch?v=dY__x2dq7Sk "Satisfactory is moving to Unreal Engine 5"
[4]: https://icdevin.github.io/convey-rs/convey_rs/save/struct.Header.html