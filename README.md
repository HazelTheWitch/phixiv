# phixiv

[pixiv](https://www.pixiv.net/) embed fixer. If you run into any issues or have any suggestions to make this service better, please [make an issue](https://github.com/HazelTheWitch/phixiv/issues/new).

## How to use

Simply replace "pixiv" with "phixiv" in the url to embed properly on Discord, etc. Alternatively, if on discord simply paste the pixiv url and send `s/i/p` after, this will edit the previous message, replacing `pixiv` with `ppxiv` which will also embed properly; please note this will require the link to include the first `i` in your message.

Additionally, when embedding a post with multiple images, add `/<index>` to the end of the link to embed that image.

If you have any feature suggestions, feel free to [make an issue](https://github.com/HazelTheWitch/phixiv/issues/new).

## Path Formats

The following are the valid paths for artworks, if there is a format which isn't listed which should be embedded, please [make an issue](https://github.com/HazelTheWitch/phixiv/issues/new).

```text
/artworks/:id
/:language/artworks/:id
/artworks/:id/:index
/:language/artworks/:id/:index
```

In addition, direct image links are in these forms.

```text
/d/:id
/d/:id/:index
```
