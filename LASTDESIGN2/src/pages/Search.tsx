import { useState } from "react";
import Header from "@/components/Header";
import Footer from "@/components/Footer";
import { Card } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Search as SearchIcon, Star, Filter, SlidersHorizontal } from "lucide-react";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

const mockProducts = [
  {
    id: 1,
    name: "Premium VPN Service - 1 Year",
    category: "Software",
    price: "0.05 XMR",
    rating: 4.9,
    reviews: 234,
    image: "https://images.unsplash.com/photo-1558494949-ef010cbdcc31?w=400&h=300&fit=crop",
    vendor: "SecureNet",
  },
  {
    id: 2,
    name: "Hardware Wallet Pro",
    category: "Hardware",
    price: "0.12 XMR",
    rating: 5.0,
    reviews: 187,
    image: "https://images.unsplash.com/photo-1639762681485-074b7f938ba0?w=400&h=300&fit=crop",
    vendor: "CryptoSafe",
  },
  {
    id: 3,
    name: "Cloud Storage 1TB Encrypted",
    category: "Digital Services",
    price: "0.08 XMR",
    rating: 4.8,
    reviews: 321,
    image: "https://images.unsplash.com/photo-1597852074816-d933c7d2b988?w=400&h=300&fit=crop",
    vendor: "CloudPrivacy",
  },
  {
    id: 4,
    name: "Privacy Tools Bundle",
    category: "Software",
    price: "0.15 XMR",
    rating: 4.9,
    reviews: 156,
    image: "https://images.unsplash.com/photo-1555949963-ff9fe0c870eb?w=400&h=300&fit=crop",
    vendor: "PrivacyFirst",
  },
  {
    id: 5,
    name: "Secure Messenger License",
    category: "Software",
    price: "0.03 XMR",
    rating: 4.7,
    reviews: 445,
    image: "https://images.unsplash.com/photo-1611746872915-64382b5c76da?w=400&h=300&fit=crop",
    vendor: "SecureChat",
  },
  {
    id: 6,
    name: "Password Manager Pro",
    category: "Software",
    price: "0.04 XMR",
    rating: 4.9,
    reviews: 567,
    image: "https://images.unsplash.com/photo-1633265486064-086b219458ec?w=400&h=300&fit=crop",
    vendor: "PassKeeper",
  },
];

const Search = () => {
  const [searchQuery, setSearchQuery] = useState("");
  const [category, setCategory] = useState("all");
  const [sortBy, setSortBy] = useState("relevance");

  const filteredProducts = mockProducts.filter(product => {
    const matchesSearch = product.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
                         product.vendor.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesCategory = category === "all" || product.category === category;
    return matchesSearch && matchesCategory;
  });

  const sortedProducts = [...filteredProducts].sort((a, b) => {
    if (sortBy === "price-low") return parseFloat(a.price) - parseFloat(b.price);
    if (sortBy === "price-high") return parseFloat(b.price) - parseFloat(a.price);
    if (sortBy === "rating") return b.rating - a.rating;
    return 0;
  });

  return (
    <div className="min-h-screen flex flex-col">
      <Header />
      
      <main className="flex-1">
        {/* Search Header */}
        <section className="bg-gradient-to-br from-coral/5 via-background to-sky/5 py-12 border-b">
          <div className="container mx-auto px-4">
            <div className="max-w-4xl mx-auto space-y-6">
              <h1 className="text-4xl font-bold text-center">
                Search <span className="text-coral">Products</span>
              </h1>
              
              <div className="relative">
                <SearchIcon className="absolute left-4 top-1/2 -translate-y-1/2 h-5 w-5 text-muted-foreground" />
                <Input
                  type="text"
                  placeholder="Search for products, vendors, or categories..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="pl-12 h-14 text-lg border-2 focus-visible:ring-coral"
                />
              </div>

              {/* Filters */}
              <div className="flex flex-wrap gap-4">
                <div className="flex-1 min-w-[200px]">
                  <Select value={category} onValueChange={setCategory}>
                    <SelectTrigger className="h-12 border-2">
                      <Filter className="h-4 w-4 mr-2" />
                      <SelectValue placeholder="Category" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="all">All Categories</SelectItem>
                      <SelectItem value="Software">Software</SelectItem>
                      <SelectItem value="Hardware">Hardware</SelectItem>
                      <SelectItem value="Digital Services">Digital Services</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div className="flex-1 min-w-[200px]">
                  <Select value={sortBy} onValueChange={setSortBy}>
                    <SelectTrigger className="h-12 border-2">
                      <SlidersHorizontal className="h-4 w-4 mr-2" />
                      <SelectValue placeholder="Sort by" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="relevance">Relevance</SelectItem>
                      <SelectItem value="rating">Highest Rated</SelectItem>
                      <SelectItem value="price-low">Price: Low to High</SelectItem>
                      <SelectItem value="price-high">Price: High to Low</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>
            </div>
          </div>
        </section>

        {/* Results */}
        <section className="py-12">
          <div className="container mx-auto px-4">
            <div className="mb-6">
              <p className="text-muted-foreground">
                Found <span className="font-bold text-foreground">{sortedProducts.length}</span> products
              </p>
            </div>

            {sortedProducts.length === 0 ? (
              <div className="text-center py-20">
                <SearchIcon className="h-16 w-16 text-muted-foreground mx-auto mb-4" />
                <h3 className="text-2xl font-bold mb-2">No products found</h3>
                <p className="text-muted-foreground">Try adjusting your search or filters</p>
              </div>
            ) : (
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
                {sortedProducts.map((product, index) => (
                  <Card
                    key={product.id}
                    className="group overflow-hidden border-none shadow-md hover:shadow-xl transition-all duration-300 hover:scale-105 animate-fade-in cursor-pointer"
                    style={{ animationDelay: `${index * 50}ms` }}
                  >
                    <div className="relative aspect-[4/3] overflow-hidden bg-muted">
                      <img
                        src={product.image}
                        alt={product.name}
                        className="w-full h-full object-cover group-hover:scale-110 transition-transform duration-500"
                      />
                      <div className="absolute top-3 right-3 bg-coral text-white px-3 py-1 rounded-full text-xs font-bold">
                        {product.category}
                      </div>
                    </div>
                    
                    <div className="p-5">
                      <p className="text-xs text-muted-foreground mb-1">by {product.vendor}</p>
                      <h3 className="font-bold text-lg mb-2 line-clamp-2">{product.name}</h3>
                      
                      <div className="flex items-center gap-2 mb-3">
                        <div className="flex items-center gap-1">
                          <Star className="h-4 w-4 fill-sunshine text-sunshine" />
                          <span className="text-sm font-medium">{product.rating}</span>
                        </div>
                        <span className="text-xs text-muted-foreground">
                          ({product.reviews})
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
            )}
          </div>
        </section>
      </main>

      <Footer />
    </div>
  );
};

export default Search;
