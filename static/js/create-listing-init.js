// Initialize components for create listing page
document.addEventListener('DOMContentLoaded', function() {
    console.log('🚀 Initializing create listing page components...');

    // Check if XMR converter function exists
    if (typeof initXmrConverter === 'undefined') {
        console.error('❌ initXmrConverter function not found!');
        return;
    }

    // Check if input elements exist
    const xmrInput = document.getElementById('price-xmr-human');
    const atomicInput = document.getElementById('price_xmr');

    console.log('🔍 XMR Input:', xmrInput);
    console.log('🔍 Atomic Input:', atomicInput);

    if (!xmrInput || !atomicInput) {
        console.error('❌ Input elements not found!');
        return;
    }

    // Initialize XMR converter
    console.log('✅ Initializing XMR converter...');
    initXmrConverter('price-xmr-human', 'price_xmr', {
        onChange: function(data) {
            console.log('💰 Price updated:', data);
            // Trigger preview update when price changes
            if (window.listingPreview) {
                window.listingPreview.updatePreview();
            }
        }
    });

    // Initialize listing preview (vendorName is set via data attribute)
    const previewContainer = document.getElementById('listing-preview');
    if (previewContainer) {
        const vendorName = previewContainer.dataset.vendorName;
        if (vendorName) {
            console.log('✅ Initializing listing preview for vendor:', vendorName);
            const listingPreview = new ListingPreview({
                formId: 'create-listing-form',
                previewContainerId: 'listing-preview',
                vendorName: vendorName
            });

            // Export globally for debugging and integration
            window.listingPreview = listingPreview;
        }
    }

    // Initialize Lucide icons
    if (typeof lucide !== 'undefined') {
        console.log('✅ Initializing Lucide icons...');
        lucide.createIcons();
    }

    console.log('✨ Create listing page initialization complete!');
});
