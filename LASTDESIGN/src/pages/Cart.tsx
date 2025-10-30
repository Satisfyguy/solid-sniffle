import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";
import { Minus, Plus, Trash2, ShoppingBag, ArrowLeft } from "lucide-react";
import { Link } from "react-router-dom";

interface CartItem {
  id: string;
  title: string;
  price: string;
  image: string;
  quantity: number;
}

const Cart = () => {
  const [cartItems, setCartItems] = useState<CartItem[]>([
    {
      id: "1",
      title: "Premium Wireless Headphones",
      price: "0.24",
      image: "https://images.unsplash.com/photo-1505740420928-5e560c06d30e?w=400&q=80",
      quantity: 1,
    },
    {
      id: "2",
      title: "Smart Fitness Watch",
      price: "0.18",
      image: "https://images.unsplash.com/photo-1523275335684-37898b6baf30?w=400&q=80",
      quantity: 2,
    },
  ]);

  const updateQuantity = (id: string, delta: number) => {
    setCartItems(items =>
      items.map(item =>
        item.id === id
          ? { ...item, quantity: Math.max(1, item.quantity + delta) }
          : item
      )
    );
  };

  const removeItem = (id: string) => {
    setCartItems(items => items.filter(item => item.id !== id));
  };

  const subtotal = cartItems.reduce(
    (sum, item) => sum + parseFloat(item.price) * item.quantity,
    0
  );

  const escrowFee = subtotal * 0.02; // 2% escrow fee
  const total = subtotal + escrowFee;

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="border-b bg-background/95 backdrop-blur">
        <div className="container flex h-16 items-center px-4 md:px-8">
          <Link to="/" className="flex items-center gap-2">
            <div className="text-2xl font-bold tracking-tight text-primary">NEXUS</div>
          </Link>
        </div>
      </header>

      <div className="container px-4 md:px-8 py-8 md:py-16">
        <div className="mb-8">
          <Link to="/" className="inline-flex items-center gap-2 text-muted-foreground hover:text-primary transition-colors">
            <ArrowLeft className="h-4 w-4" />
            Continue Shopping
          </Link>
        </div>

        {cartItems.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-16 text-center">
            <div className="h-24 w-24 rounded-full bg-muted flex items-center justify-center mb-6">
              <ShoppingBag className="h-12 w-12 text-muted-foreground" />
            </div>
            <h2 className="text-3xl font-bold mb-2">Your Cart is Empty</h2>
            <p className="text-muted-foreground mb-8">Add some products to get started</p>
            <Link to="/">
              <Button size="lg">Browse Products</Button>
            </Link>
          </div>
        ) : (
          <div className="grid lg:grid-cols-3 gap-8">
            {/* Cart Items */}
            <div className="lg:col-span-2 space-y-4">
              <h1 className="text-3xl font-bold mb-6">Shopping Cart</h1>

              {cartItems.map((item) => (
                <Card key={item.id}>
                  <CardContent className="p-6">
                    <div className="flex gap-4">
                      <img
                        src={item.image}
                        alt={item.title}
                        className="h-24 w-24 rounded-lg object-cover"
                      />
                      <div className="flex-1">
                        <h3 className="font-semibold text-lg mb-2">{item.title}</h3>
                        <p className="text-2xl font-bold text-primary">
                          {item.price} XMR
                        </p>
                      </div>
                      <div className="flex flex-col items-end justify-between">
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => removeItem(item.id)}
                          className="text-destructive hover:text-destructive"
                        >
                          <Trash2 className="h-4 w-4" />
                        </Button>
                        <div className="flex items-center gap-2">
                          <Button
                            variant="outline"
                            size="icon"
                            className="h-8 w-8"
                            onClick={() => updateQuantity(item.id, -1)}
                          >
                            <Minus className="h-3 w-3" />
                          </Button>
                          <span className="w-8 text-center font-medium">
                            {item.quantity}
                          </span>
                          <Button
                            variant="outline"
                            size="icon"
                            className="h-8 w-8"
                            onClick={() => updateQuantity(item.id, 1)}
                          >
                            <Plus className="h-3 w-3" />
                          </Button>
                        </div>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>

            {/* Order Summary */}
            <div className="lg:col-span-1">
              <Card className="sticky top-24">
                <CardContent className="p-6 space-y-6">
                  <h2 className="text-2xl font-bold">Order Summary</h2>

                  <div className="space-y-4">
                    <div className="flex justify-between text-muted-foreground">
                      <span>Subtotal</span>
                      <span className="font-medium">{subtotal.toFixed(4)} XMR</span>
                    </div>

                    <div className="flex justify-between text-muted-foreground">
                      <span>Escrow Fee (2%)</span>
                      <span className="font-medium">{escrowFee.toFixed(4)} XMR</span>
                    </div>

                    <Separator />

                    <div className="flex justify-between text-xl font-bold">
                      <span>Total</span>
                      <span className="text-primary">{total.toFixed(4)} XMR</span>
                    </div>

                    <div className="text-sm text-muted-foreground bg-muted p-4 rounded-lg">
                      <p className="font-medium mb-2">🔒 Secure 2/3 Multisig Escrow</p>
                      <p className="text-xs">
                        Your payment will be held securely until you confirm delivery. 
                        Your keys, your control.
                      </p>
                    </div>
                  </div>

                  <Button className="w-full" size="lg">
                    Proceed to Checkout
                  </Button>

                  <div className="text-center">
                    <Link
                      to="/"
                      className="text-sm text-muted-foreground hover:text-primary transition-colors"
                    >
                      Continue Shopping
                    </Link>
                  </div>
                </CardContent>
              </Card>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default Cart;
