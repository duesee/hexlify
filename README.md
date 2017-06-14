Command-line application implementing Python's binascii.{un,}hexlify. Use it as you would use base64.

```
$ hexlify --help
hexlify 0.0.1

Perform bytes-to-hexstring conversion and vice-versa as implemented
in Python's binascii.{un,}hexlify. Read from stdin if <file> is "-"
or not specified. Whitespace is ignored during decoding.

Usage: monitor [options] [<file>]

Options:
  -d --decode          Decode stream.
  -i --ignore-garbage  Ignore non-hex values.
  -h --help            Show this help screen.
```

Examples:

```
$ hexlify /dev/urandom
23B66A15A74230AF7D4B4B57C24024C7E3747A21E599D682DA7E0D50AB41B39DF178A20DCF97A523CE8CEAD928299390B0C7065C588F37AD0ACFB0F66D70B1A27E32D1CA4B5B41C24A3E9C7F9CCEB9FA604AB9FCF4069A21FAB99D0EF6311BD8A084AF0375B0DC413592A5F5E99059CC1D98F5C5455407EF3867333B560581B6B0491
```

```
$ echo -n "00FF22" | hexlify -d | xxd -g1
00000000: 00 ff 22                                         .."
```

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
