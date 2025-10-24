document.addEventListener('DOMContentLoaded', () => {
    const form = document.getElementById('create-listing-form');
    if (!form) return;

    form.addEventListener('submit', async function(event) {
        event.preventDefault();

        const formData = new FormData(form);
        const resultDiv = document.getElementById('listing-result');
        const submitButton = form.querySelector('button[type="submit"]');

        const csrfToken = formData.get('csrf_token');

        const data = {
            title: formData.get('title'),
            description: formData.get('description'),
            price_xmr: parseInt(formData.get('price_xmr'), 10),
            stock: parseInt(formData.get('stock'), 10),
        };

        resultDiv.innerHTML = '<p style="color: #888;">CREATING...</p>';
        submitButton.disabled = true;

        try {
            const response = await fetch('/api/listings', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'X-CSRF-Token': csrfToken,
                },
                body: JSON.stringify(data),
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
                resultDiv.innerHTML = `<p style="color: #22c55e;">SUCCESS! Listing created.</p><pre>${JSON.stringify(responseData, null, 2)}</pre>`;
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
