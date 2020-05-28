# dicomrle-rs
DICOM RLE Codec written in Rust

## Goal

Full implementation of a DICOM RLE CODEC delivered as a rust library and a webassembly module
with javascript bindings.

## Status

Actively being developed (May 28, 2020)

## Tasks

- [x] Implement Decoder
- [ ] Implement Encoder
- [ ] Create WebAssembly Build
- [ ] Create JS Glue for WebAssembly Build
- [ ] Create NodeJS Demo Application using WebAssembly Build
- [ ] Create HTML/JS Demo Application using WebAssembly Build
- [ ] Create Performance Bechmarks

## Relevant Links

* [DICOM Standard on RLE Encoding](http://dicom.nema.org/medical/Dicom/current/output/chtml/part05/sect_8.2.2.html)
* [CornerstoneJS JavaScript Source](https://github.com/cornerstonejs/cornerstoneWADOImageLoader/blob/master/src/shared/decoders/decodeRLE.js)
* [GDCM C++ Source](https://github.com/malaterre/GDCM/blob/master/Source/MediaStorageAndFileFormat/gdcmRLECodec.cxx)
* [DCMTK C++ Source](https://github.com/DCMTK/dcmtk/blob/master/dcmdata/libsrc/dcrleccd.cc)
* [ClearCanvas C# Source](https://github.com/ClearCanvas/ClearCanvas/blob/master/Dicom/Codec/Rle/DicomRleCodec.cs)

## Decoder Design Notes

### Presizing the decoded buffer

The decode() api requires that the caller presize the decode buffer to be the
expected size of the decoded image.  This was done so the decoder could
properly detect incomplete decodes.  Since this decoder is designed to
decode DICOM RLE Images, the caller should have access to the DICOM Header
which contains the attributes needed to calculate the size of the decoded
image buffer (specifically - rows, columns, bits allocated and samples per 
pixel).

### Stream Support

Support for decoding from a Read stream is not currently supported.  While
technically possible, the decode logic is simplified by knowing the full
size of the encoded bitstream up front.

Support for decoding to a Write stream is not currently supported.  While
technically possible, the decode logic is simplified by knowing the full
size of the decoded image up front.  

Note that you cannot properly support Read and Write streams at the same time
without buffering one of them due to the deinterleaving of pixel data accross
segments that is part of the DICOM RLE.  This fact reduces the value of having
a full stream based API which could mislead the caller into thinking it has
low memory overhead when it can't due to the variable sized internal buffering
that would be required.

### Bounds Checking

The decoder currently performs a lot of manual bounds checking, some of which
may be redundant to that being done by Vector or unnecessary with 
different logic.  

It feels like an iterator based design would simplify, improve performance 
and open the door for parallelization.  This is an area to consider in the
future.