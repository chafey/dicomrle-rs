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
