# Shell Integration Config

Contains the bootstrap scripts that Haneulchi shells source to emit structured markers for:

- current working directory
- branch hint
- command boundary
- command exit code

Marker format:

- prefix: ASCII unit separator + `HC|`
- examples:
  - `\037HC|cwd|/tmp/demo`
  - `\037HC|command|npm test`
  - `\037HC|exit|0`

`hc-runtime` strips these marker lines from operator-visible output and stores them as structured session metadata.
