import { Package, Code, Cpu, FileText, Palette, Briefcase } from "lucide-react";
import { Card } from "@/components/ui/card";

const categories = [
  {
    name: "Software",
    icon: Code,
    bgColor: "bg-coral",
    textColor: "text-coral-foreground",
    items: "2,543 items",
  },
  {
    name: "Digital Services",
    icon: Briefcase,
    bgColor: "bg-sunshine",
    textColor: "text-sunshine-foreground",
    items: "1,892 items",
  },
  {
    name: "Secure Hardware",
    icon: Cpu,
    bgColor: "bg-mint",
    textColor: "text-mint-foreground",
    items: "867 items",
  },
  {
    name: "Documents",
    icon: FileText,
    bgColor: "bg-sky",
    textColor: "text-sky-foreground",
    items: "3,421 items",
  },
  {
    name: "Design Assets",
    icon: Palette,
    bgColor: "bg-coral",
    textColor: "text-coral-foreground",
    items: "1,234 items",
  },
  {
    name: "Physical Goods",
    icon: Package,
    bgColor: "bg-mint",
    textColor: "text-mint-foreground",
    items: "4,567 items",
  },
];

const Categories = () => {
  return (
    <section className="py-20 bg-secondary/30">
      <div className="container mx-auto px-4">
        <div className="text-center mb-12">
          <h2 className="text-4xl font-bold mb-4">Shop by Category</h2>
          <p className="text-muted-foreground text-lg">
            Explore our diverse marketplace with complete privacy
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {categories.map((category, index) => {
            const Icon = category.icon;
            
            return (
              <Card
                key={index}
                className={`group relative overflow-hidden border-none ${category.bgColor} ${category.textColor} p-8 cursor-pointer transition-all duration-300 hover:scale-105 hover:shadow-xl animate-fade-in`}
                style={{
                  animationDelay: `${index * 100}ms`,
                }}
              >
                <div className="flex items-start justify-between mb-4">
                  <div className="p-3 bg-white/20 rounded-xl backdrop-blur-sm">
                    <Icon className="h-8 w-8" />
                  </div>
                  <span className="text-sm font-medium opacity-90">{category.items}</span>
                </div>
                
                <h3 className="text-2xl font-bold mb-2">{category.name}</h3>
                <p className="text-sm opacity-90">Browse collection →</p>
                
                <div className="absolute bottom-0 right-0 w-32 h-32 bg-white/10 rounded-full translate-x-16 translate-y-16 group-hover:scale-150 transition-transform duration-500"></div>
              </Card>
            );
          })}
        </div>
      </div>
    </section>
  );
};

export default Categories;
