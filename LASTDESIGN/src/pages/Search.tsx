import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import ProductCard from "@/components/ProductCard";
import { Search as SearchIcon, SlidersHorizontal, X } from "lucide-react";
import { Link } from "react-router-dom";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Card, CardContent } from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";

const Search = () => {
  const [searchQuery, setSearchQuery] = useState("");
  const [showFilters, setShowFilters] = useState(false);
  const [selectedCategory, setSelectedCategory] = useState("all");
  const [priceRange, setPriceRange] = useState("all");

  // Mock search results
  const searchResults = [
    {
      title: "Premium Wireless Headphones",
      price: "0.24",
      image: "https://images.unsplash.com/photo-1505740420928-5e560c06d30e?w=800&q=80",
      category: "Audio",
      featured: true,
    },
    {
      title: "Smart Fitness Watch",
      price: "0.18",
      image: "https://images.unsplash.com/photo-1523275335684-37898b6baf30?w=800&q=80",
      category: "Wearables",
    },
    {
      title: "Ultrabook Laptop Pro",
      price: "1.85",
      image: "https://images.unsplash.com/photo-1496181133206-80ce9b88a853?w=800&q=80",
      category: "Devices",
      featured: true,
    },
    {
      title: "Gaming Console Bundle",
      price: "0.95",
      image: "https://images.unsplash.com/photo-1486401899868-0e435ed85128?w=800&q=80",
      category: "Gaming",
    },
    {
      title: "Mechanical Keyboard RGB",
      price: "0.12",
      image: "https://images.unsplash.com/photo-1587829741301-dc798b83add3?w=800&q=80",
      category: "Devices",
    },
    {
      title: "Wireless Gaming Mouse",
      price: "0.08",
      image: "https://images.unsplash.com/photo-1527864550417-7fd91fc51a46?w=800&q=80",
      category: "Gaming",
    },
  ];

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

      <div className="container px-4 md:px-8 py-8">
        {/* Search Bar */}
        <div className="max-w-3xl mx-auto mb-8 space-y-4">
          <div className="relative">
            <SearchIcon className="absolute left-4 top-1/2 -translate-y-1/2 h-5 w-5 text-muted-foreground" />
            <Input
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search for products, categories, vendors..."
              className="pl-12 pr-12 h-14 text-lg"
            />
            {searchQuery && (
              <button
                onClick={() => setSearchQuery("")}
                className="absolute right-4 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
              >
                <X className="h-5 w-5" />
              </button>
            )}
          </div>

          <div className="flex items-center justify-between">
            <p className="text-muted-foreground">
              Found <span className="font-semibold text-foreground">{searchResults.length}</span> results
            </p>
            <Button
              variant="outline"
              onClick={() => setShowFilters(!showFilters)}
              className="gap-2"
            >
              <SlidersHorizontal className="h-4 w-4" />
              Filters
            </Button>
          </div>
        </div>

        {/* Filters */}
        {showFilters && (
          <Card className="max-w-3xl mx-auto mb-8 animate-slide-up">
            <CardContent className="p-6">
              <div className="grid md:grid-cols-3 gap-6">
                <div className="space-y-2">
                  <label className="text-sm font-medium">Category</label>
                  <Select value={selectedCategory} onValueChange={setSelectedCategory}>
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="all">All Categories</SelectItem>
                      <SelectItem value="audio">Audio</SelectItem>
                      <SelectItem value="wearables">Wearables</SelectItem>
                      <SelectItem value="devices">Devices</SelectItem>
                      <SelectItem value="gaming">Gaming</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div className="space-y-2">
                  <label className="text-sm font-medium">Price Range (XMR)</label>
                  <Select value={priceRange} onValueChange={setPriceRange}>
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="all">All Prices</SelectItem>
                      <SelectItem value="0-0.5">0 - 0.5 XMR</SelectItem>
                      <SelectItem value="0.5-1">0.5 - 1 XMR</SelectItem>
                      <SelectItem value="1-2">1 - 2 XMR</SelectItem>
                      <SelectItem value="2+">2+ XMR</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div className="space-y-2">
                  <label className="text-sm font-medium">Sort By</label>
                  <Select defaultValue="relevance">
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="relevance">Relevance</SelectItem>
                      <SelectItem value="price-low">Price: Low to High</SelectItem>
                      <SelectItem value="price-high">Price: High to Low</SelectItem>
                      <SelectItem value="newest">Newest First</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>

              <Separator className="my-6" />

              <div className="flex items-center justify-between">
                <div className="flex gap-2">
                  {selectedCategory !== "all" && (
                    <Button
                      variant="secondary"
                      size="sm"
                      onClick={() => setSelectedCategory("all")}
                      className="gap-2"
                    >
                      Category: {selectedCategory}
                      <X className="h-3 w-3" />
                    </Button>
                  )}
                  {priceRange !== "all" && (
                    <Button
                      variant="secondary"
                      size="sm"
                      onClick={() => setPriceRange("all")}
                      className="gap-2"
                    >
                      Price: {priceRange}
                      <X className="h-3 w-3" />
                    </Button>
                  )}
                </div>
                {(selectedCategory !== "all" || priceRange !== "all") && (
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => {
                      setSelectedCategory("all");
                      setPriceRange("all");
                    }}
                  >
                    Clear All
                  </Button>
                )}
              </div>
            </CardContent>
          </Card>
        )}

        {/* Search Results */}
        <div className="grid sm:grid-cols-2 lg:grid-cols-4 gap-6">
          {searchResults.map((product, index) => (
            <ProductCard key={index} {...product} />
          ))}
        </div>

        {/* Empty State */}
        {searchResults.length === 0 && (
          <div className="text-center py-16">
            <div className="h-24 w-24 rounded-full bg-muted flex items-center justify-center mx-auto mb-6">
              <SearchIcon className="h-12 w-12 text-muted-foreground" />
            </div>
            <h2 className="text-3xl font-bold mb-2">No Results Found</h2>
            <p className="text-muted-foreground mb-8">Try adjusting your search or filters</p>
            <Button onClick={() => setSearchQuery("")}>Clear Search</Button>
          </div>
        )}
      </div>
    </div>
  );
};

export default Search;
