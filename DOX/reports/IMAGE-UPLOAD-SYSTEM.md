# Image Upload System for Monero Marketplace

## Overview

The image upload system allows vendors to upload product images that are stored on IPFS (InterPlanetary File System) and served through the marketplace server. This ensures decentralized, censorship-resistant image storage while maintaining good user experience.

## Architecture

### Components

1. **Frontend (JavaScript)**
   - Drag-and-drop upload interface
   - Progress indicators
   - Image preview and management
   - File validation (type, size)

2. **Backend (Rust)**
   - Multipart form data handling
   - Image validation (magic bytes)
   - IPFS integration
   - Database storage of IPFS CIDs

3. **IPFS Storage**
   - Decentralized image storage
   - Content-addressed storage (CIDs)
   - Tor integration for privacy

4. **Database**
   - Stores IPFS CIDs as JSON array
   - Links images to listings

## API Endpoints

### Upload Images
```
POST /api/listings/{id}/images
Content-Type: multipart/form-data

Fields:
- images: File[] (multiple image files)
```

**Response:**
```json
{
  "message": "Images uploaded successfully",
  "image_count": 3
}
```

### Get Image
```
GET /api/listings/{id}/images/{cid}
```

**Response:**
- Content-Type: image/jpeg, image/png, or image/gif
- Binary image data

## Database Schema

### Listings Table
```sql
ALTER TABLE listings ADD COLUMN images_ipfs_cids TEXT DEFAULT '[]';
```

**Format:** JSON array of IPFS CIDs
```json
["QmHash1", "QmHash2", "QmHash3"]
```

## File Validation

### Supported Formats
- JPEG (magic bytes: `FF D8 FF`)
- PNG (magic bytes: `89 50 4E 47`)
- GIF (magic bytes: `47 49 46 38`)

### Limits
- Maximum 10 images per listing
- Maximum 5MB per image
- Total upload size: 50MB per request

## Security Features

### Tor Integration
- All IPFS traffic routed through Tor SOCKS5 proxy
- Prevents IP address leaks
- Configurable via `IPFS_USE_TOR` environment variable

### Input Validation
- File type validation using magic bytes
- File size limits
- Maximum file count limits
- Ownership verification (only listing owner can upload)

### Error Handling
- Graceful error responses
- No sensitive information in error messages
- Proper HTTP status codes

## Frontend Features

### Upload Interface
- Drag-and-drop zone
- Click to select files
- Visual progress indicators
- Real-time validation feedback

### Image Management
- Thumbnail previews
- Remove image functionality
- Responsive grid layout
- Modal image viewer

### User Experience
- Instant feedback on upload status
- Error messages with clear explanations
- Loading states and progress bars
- Mobile-friendly interface

## Usage

### For Vendors

1. **Create Listing**
   - Fill out listing details
   - Images will be uploaded after listing creation

2. **Edit Listing**
   - Add/remove images using drag-and-drop
   - Images are uploaded immediately

3. **View Images**
   - Images display in responsive grid
   - Click to view full-size in modal

### For Buyers

1. **Browse Listings**
   - Images display as thumbnails
   - Click to view full-size images

2. **Listing Details**
   - High-quality image gallery
   - Zoom functionality

## Development

### Prerequisites
- IPFS daemon running on localhost:5001
- Tor proxy (optional, for production)
- Rust toolchain
- Node.js (for frontend assets)

### Setup

1. **Start IPFS**
   ```bash
   ipfs init
   ipfs daemon
   ```

2. **Configure Tor (Production)**
   ```bash
   export IPFS_USE_TOR=true
   ```

3. **Start Server**
   ```bash
   cargo run -p server --bin server
   ```

### Testing

Run the test script:
```bash
./test-image-upload.sh
```

### File Structure

```
server/src/handlers/listings.rs    # API endpoints
static/js/upload-images.js         # Frontend JavaScript
templates/listings/                # HTML templates
server/src/ipfs/client.rs          # IPFS integration
```

## Configuration

### Environment Variables

- `IPFS_USE_TOR`: Enable Tor proxy for IPFS (default: false)
- `IPFS_API_URL`: IPFS API endpoint (default: http://localhost:5001/api/v0)
- `IPFS_GATEWAY_URL`: IPFS Gateway URL (default: http://localhost:8080/ipfs)

### IPFS Settings

The system uses a local IPFS node by default. For production:

1. **Infura IPFS**
   ```rust
   let client = IpfsClient::new_infura(project_id, project_secret)?;
   ```

2. **Custom IPFS Node**
   ```rust
   let client = IpfsClient::new(api_url, gateway_url)?;
   ```

## Troubleshooting

### Common Issues

1. **IPFS Not Running**
   - Error: "IPFS service unavailable"
   - Solution: Start IPFS daemon

2. **Upload Fails**
   - Check file size (max 5MB)
   - Check file type (JPEG/PNG/GIF only)
   - Check authentication

3. **Images Not Displaying**
   - Check IPFS gateway accessibility
   - Verify CID format in database
   - Check network connectivity

### Debug Mode

Enable debug logging:
```bash
RUST_LOG=debug cargo run -p server --bin server
```

## Security Considerations

### Privacy
- All IPFS traffic goes through Tor
- No IP address logging
- Decentralized storage prevents censorship

### Data Integrity
- Content-addressed storage (CIDs)
- Immutable image storage
- Cryptographic verification

### Access Control
- Only listing owners can upload images
- Authentication required for all operations
- Rate limiting on uploads

## Performance

### Optimization
- Connection pooling for IPFS
- Retry logic with exponential backoff
- Efficient multipart handling
- Client-side image compression (future)

### Monitoring
- Upload success/failure rates
- IPFS node health
- Storage usage metrics
- Response times

## Future Enhancements

### Planned Features
- Image compression before upload
- Thumbnail generation
- Batch upload progress
- Image editing tools
- CDN integration

### Scalability
- Multiple IPFS nodes
- Load balancing
- Caching layer
- Database optimization

## Support

For issues or questions:
1. Check the troubleshooting section
2. Review server logs
3. Test with the provided test script
4. Check IPFS node status

---

**Note:** This system is designed for the Monero Marketplace and follows the project's security-first approach with Tor integration and decentralized storage.
