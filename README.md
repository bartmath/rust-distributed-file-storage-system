# Distributed file storage system

This project implements a highly optimized distributed file storage system in Rust, designed to scale up to 10^7 files of 64 MB each. The system architecture features:
- A single centralized metadata server managing file metadata and chunk locations
- Multiple racks hosting chunkservers (storage servers) that store encrypted file chunks
- Secure client-side encryption and decryption, ensuring that only file owners can read file contents

Communication is based on QUIC using Tokio for asynchronous, high-performance networking. The system supports client login, file upload, and download functionalities.

## System Worklow
### Uploading files
- Client splits files into chunks (max 64 MB each) and encrypts each chunk locally
- Client requests an upload plan from the metadata server specifying which chunkserver each chunk should be uploaded to
- Client uploads encrypted chunks to the designated chunkservers
- (optional) Chunkservers replicate chunks across different servers for fault tolerance
- Metadata server finalizes the file upload, updating metadata with chunk locations

### Downloading files
- Client requests chunk locations for a given file from the metadata server
- Client downloads encrypted chunks from the respective chunkservers
- Client decrypts and reassembles the file locally

## Plan

### Phase 1:
- Storage server:
    * Discover metadata server using a heartbeat protocol
    * Store chunks reliably on disk
    * Expose upload/download chunk API
- Metadata server:
    * Assume (for now) that all files are stored in a single folder
    * Track chunk locations per file for download requests
### Phase 2:
- Metadata server:
    * Support a flat directory (single folder) for all files
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
    * Support chunks rebalancing acorss chunkservers & racks
- Metadata server:  
    * Integrate rdedup Rust library for deduplication with user-configurable rules to avoid duplicate file storage
- Client:
    * Add caching and lease mechanism to improve performance and consistency
    
## Libraries:
- tokio - for highly concurrent servers
- quinn - for communication with servers using QUIC protocol