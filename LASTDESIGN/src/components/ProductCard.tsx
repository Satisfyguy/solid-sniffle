import { Card } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ShoppingCart } from "lucide-react";

interface ProductCardProps {
  title: string;
  price: string;
  image: string;
  category: string;
  featured?: boolean;
}

const ProductCard = ({ title, price, image, category, featured }: ProductCardProps) => {
  return (
    <Card className="group overflow-hidden border-0 shadow-md hover:shadow-xl transition-all">
      <div className="aspect-square bg-muted relative overflow-hidden">
        <img 
          src={image} 
          alt={title}
          className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
        />
        {featured && (
          <Badge className="absolute top-4 left-4 bg-accent">Featured</Badge>
        )}
      </div>
      <div className="p-5 space-y-3">
        <div className="space-y-1">
          <p className="text-xs text-muted-foreground uppercase tracking-wider">{category}</p>
          <h3 className="font-semibold text-lg line-clamp-2">{title}</h3>
        </div>
        <div className="flex items-center justify-between">
          <div>
            <p className="text-2xl font-bold text-primary">{price}</p>
            <p className="text-xs text-muted-foreground">XMR</p>
          </div>
          <Button size="sm" className="gap-2">
            <ShoppingCart className="h-4 w-4" />
            Add
          </Button>
        </div>
      </div>
    </Card>
  );
};

export default ProductCard;
