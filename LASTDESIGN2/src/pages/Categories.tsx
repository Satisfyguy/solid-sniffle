import { useState } from "react";
import Header from "@/components/Header";
import Footer from "@/components/Footer";
import { Card } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Search, Star, TrendingUp } from "lucide-react";

const allCategories = [
  { name: "Software", count: 2543, trending: true, color: "bg-coral" },
  { name: "Digital Services", count: 1892, trending: true, color: "bg-sunshine" },
  { name: "Secure Hardware", count: 867, trending: false, color: "bg-mint" },
  { name: "Documents", count: 3421, trending: true, color: "bg-sky" },
  { name: "Design Assets", count: 1234, trending: false, color: "bg-coral" },
  { name: "Physical Goods", count: 4567, trending: true, color: "bg-mint" },
  { name: "Privacy Tools", count: 987, trending: true, color: "bg-sky" },
  { name: "Educational Content", count: 654, trending: false, color: "bg-sunshine" },
  { name: "Entertainment", count: 2341, trending: false, color: "bg-coral" },
  { name: "Consulting", count: 432, trending: false, color: "bg-mint" },
  { name: "Development", count: 1567, trending: true, color: "bg-sky" },
  { name: "Marketing", count: 876, trending: false, color: "bg-sunshine" },
];

const Categories = () => {
  const [searchQuery, setSearchQuery] = useState("");

  const filteredCategories = allCategories.filter(category =>
    category.name.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div className="min-h-screen flex flex-col">
      <Header />
      
      <main className="flex-1">
        {/* Hero Section */}
        <section className="bg-gradient-to-br from-coral/5 via-background to-sky/5 py-20">
          <div className="container mx-auto px-4">
            <div className="max-w-3xl mx-auto text-center space-y-6">
              <h1 className="text-5xl md:text-6xl font-bold">
                Browse <span className="text-coral">Categories</span>
              </h1>
              <p className="text-lg text-muted-foreground">
                Explore thousands of products across diverse categories
              </p>
              
              <div className="relative max-w-xl mx-auto">
                <Search className="absolute left-4 top-1/2 -translate-y-1/2 h-5 w-5 text-muted-foreground" />
                <Input
                  type="text"
                  placeholder="Search categories..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="pl-12 h-14 text-lg border-2 focus-visible:ring-coral"
                />
              </div>
            </div>
          </div>
        </section>

        {/* Trending Categories */}
        <section className="py-12 bg-secondary/30">
          <div className="container mx-auto px-4">
            <div className="flex items-center gap-2 mb-6">
              <TrendingUp className="h-6 w-6 text-coral" />
              <h2 className="text-2xl font-bold">Trending Now</h2>
            </div>
            
            <div className="flex gap-3 flex-wrap">
              {allCategories
                .filter(cat => cat.trending)
                .map((category, index) => (
                  <Button
                    key={index}
                    variant="outline"
                    className="border-coral text-coral hover:bg-coral hover:text-white"
                  >
                    {category.name}
                    <Star className="h-4 w-4 ml-2 fill-current" />
                  </Button>
                ))}
            </div>
          </div>
        </section>

        {/* All Categories Grid */}
        <section className="py-20">
          <div className="container mx-auto px-4">
            <h2 className="text-3xl font-bold mb-8">All Categories</h2>
            
            {filteredCategories.length === 0 ? (
              <div className="text-center py-20">
                <p className="text-muted-foreground text-lg">No categories found matching "{searchQuery}"</p>
              </div>
            ) : (
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
                {filteredCategories.map((category, index) => (
                  <Card
                    key={index}
                    className={`group ${category.color} text-white p-6 cursor-pointer transition-all duration-300 hover:scale-105 hover:shadow-xl border-none animate-fade-in`}
                    style={{ animationDelay: `${index * 50}ms` }}
                  >
                    <div className="flex flex-col h-full justify-between">
                      <div>
                        <h3 className="text-xl font-bold mb-2">{category.name}</h3>
                        <p className="text-sm opacity-90">{category.count.toLocaleString()} items</p>
                      </div>
                      
                      {category.trending && (
                        <div className="flex items-center gap-1 mt-4 text-sm">
                          <TrendingUp className="h-4 w-4" />
                          <span>Trending</span>
                        </div>
                      )}
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

export default Categories;
