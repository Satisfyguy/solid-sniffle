import { useState } from "react";
import Header from "@/components/Header";
import Footer from "@/components/Footer";
import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Trash2, Plus, Minus, ShoppingBag, ArrowRight } from "lucide-react";
import { toast } from "sonner";

interface CartItem {
  id: number;
  name: string;
  category: string;
  price: number;
  quantity: number;
  image: string;
  vendor: string;
}

const Cart = () => {
  const [cartItems, setCartItems] = useState<CartItem[]>([
    {
      id: 1,
      name: "Premium VPN Service - 1 Year",
      category: "Software",
      price: 0.05,
      quantity: 1,
      image: "https://images.unsplash.com/photo-1558494949-ef010cbdcc31?w=200&h=150&fit=crop",
      vendor: "SecureNet",
    },
    {
      id: 2,
      name: "Hardware Wallet Pro",
      category: "Hardware",
      price: 0.12,
      quantity: 1,
      image: "https://images.unsplash.com/photo-1639762681485-074b7f938ba0?w=200&h=150&fit=crop",
      vendor: "CryptoSafe",
    },
  ]);

  const updateQuantity = (id: number, change: number) => {
    setCartItems(items =>
      items.map(item =>
        item.id === id
          ? { ...item, quantity: Math.max(1, item.quantity + change) }
          : item
      )
    );
  };

  const removeItem = (id: number) => {
    setCartItems(items => items.filter(item => item.id !== id));
    toast.success("Item removed from cart");
  };

  const total = cartItems.reduce((sum, item) => sum + item.price * item.quantity, 0);

  const handleCheckout = () => {
    toast.success("Proceeding to secure checkout with Monero...");
  };

  return (
    <div className="min-h-screen flex flex-col">
      <Header />
      
      <main className="flex-1 py-12">
        <div className="container mx-auto px-4">
          <div className="max-w-6xl mx-auto">
            <h1 className="text-4xl font-bold mb-8">
              Shopping <span className="text-coral">Cart</span>
            </h1>

            {cartItems.length === 0 ? (
              <Card className="p-12 text-center">
                <ShoppingBag className="h-16 w-16 text-muted-foreground mx-auto mb-4" />
                <h2 className="text-2xl font-bold mb-2">Your cart is empty</h2>
                <p className="text-muted-foreground mb-6">
                  Add some products to get started
                </p>
                <Button variant="hero" size="lg">
                  Browse Products
                </Button>
              </Card>
            ) : (
              <div className="grid lg:grid-cols-3 gap-8">
                {/* Cart Items */}
                <div className="lg:col-span-2 space-y-4">
                  {cartItems.map((item, index) => (
                    <Card
                      key={item.id}
                      className="p-6 animate-fade-in hover:shadow-lg transition-shadow"
                      style={{ animationDelay: `${index * 100}ms` }}
                    >
                      <div className="flex gap-6">
                        <div className="relative w-32 h-24 flex-shrink-0 rounded-lg overflow-hidden bg-muted">
                          <img
                            src={item.image}
                            alt={item.name}
                            className="w-full h-full object-cover"
                          />
                        </div>

                        <div className="flex-1 min-w-0">
                          <div className="flex justify-between items-start mb-2">
                            <div>
                              <p className="text-xs text-muted-foreground mb-1">
                                {item.category} • {item.vendor}
                              </p>
                              <h3 className="font-bold text-lg line-clamp-2">{item.name}</h3>
                            </div>
                            <Button
                              variant="ghost"
                              size="icon"
                              className="text-destructive hover:text-destructive hover:bg-destructive/10"
                              onClick={() => removeItem(item.id)}
                            >
                              <Trash2 className="h-5 w-5" />
                            </Button>
                          </div>

                          <div className="flex items-center justify-between mt-4">
                            <div className="flex items-center gap-3">
                              <Button
                                variant="outline"
                                size="icon"
                                className="h-8 w-8"
                                onClick={() => updateQuantity(item.id, -1)}
                              >
                                <Minus className="h-4 w-4" />
                              </Button>
                              <span className="font-medium w-8 text-center">
                                {item.quantity}
                              </span>
                              <Button
                                variant="outline"
                                size="icon"
                                className="h-8 w-8"
                                onClick={() => updateQuantity(item.id, 1)}
                              >
                                <Plus className="h-4 w-4" />
                              </Button>
                            </div>
                            
                            <div className="text-right">
                              <p className="text-xl font-bold text-coral">
                                {(item.price * item.quantity).toFixed(3)} XMR
                              </p>
                              {item.quantity > 1 && (
                                <p className="text-xs text-muted-foreground">
                                  {item.price.toFixed(3)} XMR each
                                </p>
                              )}
                            </div>
                          </div>
                        </div>
                      </div>
                    </Card>
                  ))}
                </div>

                {/* Order Summary */}
                <div className="lg:col-span-1">
                  <Card className="p-6 sticky top-24">
                    <h2 className="text-2xl font-bold mb-6">Order Summary</h2>
                    
                    <div className="space-y-4 mb-6">
                      <div className="flex justify-between text-muted-foreground">
                        <span>Subtotal ({cartItems.length} items)</span>
                        <span>{total.toFixed(3)} XMR</span>
                      </div>
                      
                      <div className="flex justify-between text-muted-foreground">
                        <span>Platform Fee</span>
                        <span>0.000 XMR</span>
                      </div>

                      <div className="border-t pt-4">
                        <div className="flex justify-between text-lg font-bold">
                          <span>Total</span>
                          <span className="text-coral">{total.toFixed(3)} XMR</span>
                        </div>
                      </div>
                    </div>

                    <Button
                      variant="hero"
                      size="lg"
                      className="w-full group"
                      onClick={handleCheckout}
                    >
                      Proceed to Checkout
                      <ArrowRight className="h-5 w-5 group-hover:translate-x-1 transition-transform" />
                    </Button>

                    <div className="mt-6 p-4 bg-secondary/50 rounded-lg">
                      <p className="text-sm text-muted-foreground text-center">
                        <span className="font-medium text-foreground">Secure Payment:</span> All transactions protected by 2/3 Multisig escrow
                      </p>
                    </div>
                  </Card>
                </div>
              </div>
            )}
          </div>
        </div>
      </main>

      <Footer />
    </div>
  );
};

export default Cart;
