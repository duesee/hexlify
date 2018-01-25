### Hexlify

Command-line application implementing bytes-to-hexstring conversion and vice-versa.
When called as "unhexlify" (e.g. via a symlink), the -d flag is set automatically.

Otherwise, use it as you would use base64.

### Install via AUR (Arch Linux)

```
$ pacaur -S hexlify-git
```

### Help

```
hexlify

Perform bytes-to-hexstring conversion and vice-versa as implemented
in Python's binascii.{un,}hexlify. Read from stdin if <file> is "-"
or not specified. Whitespace is ignored during decoding.

Usage:
  hexlify [options] [<file>]
  hexlify (-h | --help)
  hexlify --version

Options:
  -d --decode          Decode stream.
  -i --ignore-garbage  Ignore non-hex values.
  -h --help            Show this screen.
  --version            Show version.

```

### Examples

```
$ echo "Hello, World" | xxd -g1
00000000: 48 65 6c 6c 6f 2c 20 57 6f 72 6c 64 0a           Hello, World.

$ echo "Hello, World" | hexlify
48656C6C6F2C20576F726C640A

$ echo "48656C6C6F2C20576F726C640A" | hexlify -d
Hello, World
```

Copy-paste works as expected. Paste some hexstring and finish with Ctrl-D.

```
$ hexlify -d | openssl asn1parse -inform der -i -dump
3031300d060960864801650304020105000420f292893ea70789183696db72b7c405cc7ea997f94b7002d05f35333a97c23084

    0:d=0  hl=2 l=  49 cons: SEQUENCE
    2:d=1  hl=2 l=  13 cons:  SEQUENCE
    4:d=2  hl=2 l=   9 prim:   OBJECT            :sha256
   15:d=2  hl=2 l=   0 prim:   NULL
   17:d=1  hl=2 l=  32 prim:  OCTET STRING
      0000 - f2 92 89 3e a7 07 89 18-36 96 db 72 b7 c4 05 cc   ...>....6..r....
      0010 - 7e a9 97 f9 4b 70 02 d0-5f 35 33 3a 97 c2 30 84   ~...Kp.._53:..0.
```
