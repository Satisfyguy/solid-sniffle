import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Star } from "lucide-react";

const products = [
  {
    name: "Premium VPN Service",
    category: "Software",
    price: "0.05 XMR",
    rating: 4.9,
    reviews: 234,
    image: "https://images.unsplash.com/photo-1558494949-ef010cbdcc31?w=400&h=300&fit=crop",
  },
  {
    name: "Secure Hardware Wallet",
    category: "Hardware",
    price: "0.12 XMR",
    rating: 5.0,
    reviews: 187,
    image: "https://images.unsplash.com/photo-1639762681485-074b7f938ba0?w=400&h=300&fit=crop",
  },
  {
    name: "Encrypted Storage",
    category: "Digital Services",
    price: "0.08 XMR",
    rating: 4.8,
    reviews: 321,
    image: "https://images.unsplash.com/photo-1597852074816-d933c7d2b988?w=400&h=300&fit=crop",
  },
  {
    name: "Privacy Tools Bundle",
    category: "Software",
    price: "0.15 XMR",
    rating: 4.9,
    reviews: 156,
    image: "https://images.unsplash.com/photo-1555949963-ff9fe0c870eb?w=400&h=300&fit=crop",
  },
];

const FeaturedProducts = () => {
  return (
    <section className="py-20 bg-secondary/30">
      <div className="container mx-auto px-4">
        <div className="text-center mb-12">
          <h2 className="text-4xl font-bold mb-4">Featured Products</h2>
          <p className="text-muted-foreground text-lg">
            Top-rated items from trusted vendors
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          {products.map((product, index) => (
            <Card
              key={index}
              className="group overflow-hidden border-none shadow-md hover:shadow-xl transition-all duration-300 hover:scale-105 animate-fade-in cursor-pointer"
              style={{ animationDelay: `${index * 100}ms` }}
            >
              <div className="relative aspect-[4/3] overflow-hidden bg-muted">
                <img
                  src={product.image}
                  alt={product.name}
                  className="w-full h-full object-cover group-hover:scale-110 transition-transform duration-500"
                />
                <div className="absolute top-3 right-3 bg-coral text-white px-3 py-1 rounded-full text-xs font-bold">
                  Featured
                </div>
              </div>
              
              <div className="p-5">
                <p className="text-xs text-muted-foreground mb-2">{product.category}</p>
                <h3 className="font-bold text-lg mb-2 line-clamp-1">{product.name}</h3>
                
                <div className="flex items-center gap-2 mb-3">
                  <div className="flex items-center gap-1">
                    <Star className="h-4 w-4 fill-sunshine text-sunshine" />
                    <span className="text-sm font-medium">{product.rating}</span>
                  </div>
                  <span className="text-xs text-muted-foreground">
                    ({product.reviews} reviews)
                  </span>
                </div>
                
                <div className="flex items-center justify-between">
                  <span className="text-xl font-bold text-coral">{product.price}</span>
                  <Button variant="outline" size="sm">
                    View
                  </Button>
                </div>
              </div>
            </Card>
          ))}
        </div>

        <div className="text-center mt-10">
          <Button variant="hero" size="lg">
            View All Products
          </Button>
        </div>
      </div>
    </section>
  );
};

export default FeaturedProducts;
