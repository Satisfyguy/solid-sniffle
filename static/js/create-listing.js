document.addEventListener('DOMContentLoaded', () => {
    const form = document.getElementById('create-listing-form');
    if (!form) return;

    const imageInput = document.getElementById('images');
    const imagePreview = document.getElementById('image-preview');

    // Preview images when selected
    if (imageInput) {
        imageInput.addEventListener('change', function() {
            imagePreview.innerHTML = '';
            const files = Array.from(this.files);
            
            files.forEach(file => {
                if (file.type.startsWith('image/')) {
                    const reader = new FileReader();
                    reader.onload = function(e) {
                        const img = document.createElement('img');
                        img.src = e.target.result;
                        img.style.cssText = 'width: 100%; height: 100px; object-fit: cover; border: 1px solid #2a2a2a; border-radius: 4px;';
                        imagePreview.appendChild(img);
                    };
                    reader.readAsDataURL(file);
                }
            });
        });
    }

    form.addEventListener('submit', async function(event) {
        event.preventDefault();

        const formData = new FormData(form);
        const resultDiv = document.getElementById('listing-result');
        const submitButton = form.querySelector('button[type="submit"]');

        resultDiv.innerHTML = '<p style="color: #888;">CREATING LISTING AND UPLOADING IMAGES...</p>';
        submitButton.disabled = true;

        try {
            // Use the with-images endpoint if images are present
            const hasImages = formData.getAll('images').some(file => file.size > 0);
            const endpoint = hasImages ? '/api/listings/with-images' : '/api/listings';

            const response = await fetch(endpoint, {
                method: 'POST',
                body: formData,
                credentials: 'same-origin'
            });

            if (response.ok) {
                const hxRedirect = response.headers.get('HX-Redirect');
                if (hxRedirect) {
                    resultDiv.innerHTML = '<p style="color: #22c55e;">SUCCESS! Redirecting...</p>';
                    window.location.href = hxRedirect;
                    return;
                }
            }

            const responseData = await response.json();

            if (response.ok) {
                resultDiv.innerHTML = `<p style="color: #22c55e;">SUCCESS! Listing created.</p>`;
                setTimeout(() => {
                    window.location.href = `/listings/${responseData.id}`;
                }, 1000);
            } else {
                resultDiv.innerHTML = `<p style="color: #ef4444;">ERROR</p><pre>${responseData.error || JSON.stringify(responseData)}</pre>`;
            }

        } catch (error) {
            resultDiv.innerHTML = `<p style="color: #ef4444;">NETWORK ERROR</p><pre>${error.message}</pre>`;
        } finally {
            submitButton.disabled = false;
        }
    });
});
