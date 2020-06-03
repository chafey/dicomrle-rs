# dicomrle-rs
DICOM RLE Codec written in Rust

## Goal

Full implementation of a DICOM RLE CODEC that runs in WebAssembly.

See the [dicomrle-wasm repository](https://github.com/chafey/dicomrle-wasm) for the 
WebAssembly version

## Status

Actively being developed (Jun 3, 2020)

## Tasks

- [x] Implement Decoder
- [ ] Implement Encoder
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
pixel).  Presizing the buffer also slightly improves the performance as the
buffer never has to be expanded (which may involve a copy operation)

### Stream Support

RLE decoding cannot be streamed on both the input and output simultaneously
because RLE encoded images typically contain multiple segments which must be
interleaved.  It is theoretically possible to stream either the input or output,
but not both simultaenously.  Streaming the output would be the better choice
since it is bigger, but this would complicate the decoder logic as it would
have to deal with all segments concurrently.  I decided to attempt this and
keep the decoder simpler.

