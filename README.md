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

The decode() api requires that the caller presize the decode Vector to be the
expected size of the decoded image.  This was done so the decoder could
properly detect incomplete decodes.  Since this decoder is designed to
decode DICOM RLE Images, the caller should have access to the DICOM Header
which contains the attributes needed to calculate the size of the decoded
image buffer (specifically - rows, columns, bits allocated and samples per 
pixel).  Presizing the decode Vector also improves the performance as the
vectors capacity never has to be expanded (which would involve a copy operation)

### Stream Support

RLE decoding cannot be streamed on both the input and output simultaneously
without fully buffering either the output or input because RLE encoded images 
typically contain multiple segments which must be interleaved.  It is theoretically
possible to stream either the input or output, but not both simultaenously.  
Streaming the output would be the better choice since it is bigger, but this
would complicate the decoder logic as it would have to decode all segments 
concurrently so they could be interleaved.  I have decided not to implement
streaming support as it would make things more complicated and I currently
have no need for streaming functionality.

### Use of standard library

This library makes minimal use of the standard library.  Use of the standard
library could probably be eliminated which would produce slightly smaller
WebAssembly binaries.  The public interface does not depend on the standard
library so accomplishing this is just a matter of modifying the internal
implementation.

### Unsafe code

This library does not utilize unsafe code except for the the decode_u16()
and decode_i16() functions which use it to cast the decoded buffer from
u8 to u16 or i16 to avoid an additional allocation and copy that would
otherwise be required.  If you want to avoid unsafe code and need to
decode i16 or u16 images, you can still call decode() which will work
properly even if the decoded data is i16 or u16, but you will have to convert
it yourself.