let currentQuantity = 1;

// Change main image when clicking thumbnail
function changeMainImage(thumbnail, event) {
    event.preventDefault();
    const imageCid = thumbnail.getAttribute('data-image-cid');
    const listingId = thumbnail.closest('.product-detail-layout').querySelector('input[name="listing_id"]').value;
    const mainImage = document.getElementById('main-image');

    mainImage.src = `/api/listings/${listingId}/images/${imageCid}`;

    // Update active thumbnail
    document.querySelectorAll('.product-detail-thumbnail').forEach(t => t.classList.remove('active'));
    thumbnail.classList.add('active');
}

// Open image in fullscreen modal
function openImageModal() {
    const mainImage = document.getElementById('main-image');
    const modal = document.getElementById('image-modal');
    const modalImage = document.getElementById('modal-image');

    modalImage.src = mainImage.src;
    modal.classList.remove('hidden');
}

// Close fullscreen modal
function closeImageModal() {
    document.getElementById('image-modal').classList.add('hidden');
}

// Quantity controls
function increaseQuantity() {
    currentQuantity++;
    updateQuantityDisplay();
}

function decreaseQuantity() {
    if (currentQuantity > 1) {
        currentQuantity--;
        updateQuantityDisplay();
    }
}

function updateQuantityDisplay() {
    document.getElementById('quantity-display').textContent = currentQuantity;
    document.getElementById('quantity-input').value = currentQuantity;
}

// Show/hide shipping form
function showShippingForm() {
    document.getElementById('shipping-form').style.display = 'block';
}

function hideShippingForm() {
    document.getElementById('shipping-form').style.display = 'none';
}

// Tab switching
function switchTab(event, tabId) {
    // Remove active from all buttons and contents
    document.querySelectorAll('.tab-button').forEach(btn => btn.classList.remove('active'));
    document.querySelectorAll('.tab-content').forEach(content => content.classList.remove('active'));

    // Add active to clicked button and corresponding content
    event.target.classList.add('active');
    document.getElementById(`tab-${tabId}`).classList.add('active');
}