# Distributed file storage system

This project implements a highly optimized distributed file storage system in Rust, designed to scale up to 10^7 files of 64 MB each. The system architecture features:
- A single centralized metadata server managing file metadata and chunk locations
- Multiple racks hosting chunkservers (storage servers) that store file chunks
- (Future) Secure client-side encryption and decryption, ensuring that only file owners can read file contents

Communication is based on QUIC using Tokio for asynchronous, high-performance networking. The system supports file upload, and download functionalities.

## System Workflow
### Uploading files
- Client splits files into chunks (max 64 MB each)
- Client requests an upload plan from the metadata server specifying which chunkserver each chunk should be uploaded to
- Client concurrently upload chunks to designated chunkservers

### Downloading files
- Client requests chunk locations for a given file from the metadata server
- Client downloads chunks from the respective chunkservers
- Client decrypts and reassembles the file locally

## Documentation

To view the documentation run
```
cargo doc --no-deps --open
```

## Plan

### Phase 1:
- Storage server:
    * Discover metadata server using a heartbeat protocol
    * Store chunks reliably on disk
    * Expose upload/download chunk API
- Metadata server:
    * Support a flat directory (single folder) for all files
    * Track chunk locations per file for download requests
### Phase 2:
- Metadata server:
    * Add hierarchical directory support.
    * Implement garbage collection for deleted files
- Storage server:
    * Implement chunk placement plan for uploads
- Client:
    * Implement login and authentication
    * Implement file upload with chunk splitting and encryption
    * Implement file download with metadata lookup and chunk fetching
    * List files and folders in the current directory
### Possible further additions:
- Storage server:
    * Add replication logic for fault tolerance
    * Support chunks rebalancing across chunkservers & racks
- Metadata server:
    * Integrate rdedup Rust library for deduplication with user-configurable rules to avoid duplicate file storage
- Client:
    * Add caching and lease mechanism to improve performance and consistency
    
## Key Libraries:
- `tokio` - for async, multithreaded runtime (high concurrency on servers)
- `quinn` - for communication with servers using QUIC protocol
