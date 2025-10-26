import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Shield, Star } from "lucide-react";

interface ProductCardProps {
  title: string;
  vendor: string;
  price: string;
  rating: number;
  verified: boolean;
  image: string;
}

const ProductCard = ({ title, vendor, price, rating, verified, image }: ProductCardProps) => {
  return (
    <Card className="group overflow-hidden bg-card border-2 border-border hover:border-primary transition-all duration-300">
      {/* Image */}
      <div className="relative aspect-square bg-muted overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-br from-primary/20 to-secondary/20" />
        <div className="absolute inset-0 flex items-center justify-center text-6xl font-black text-primary/10">
          ?
        </div>
        
        {/* Verified Badge */}
        {verified && (
          <Badge className="absolute top-4 right-4 bg-accent text-accent-foreground font-bold">
            <Shield className="w-3 h-3 mr-1" />
            VERIFIED
          </Badge>
        )}
      </div>

      {/* Content */}
      <div className="p-6">
        <div className="mb-4">
          <h3 className="text-xl font-black tracking-tight mb-2 line-clamp-2">
            {title}
          </h3>
          <p className="text-sm text-muted-foreground font-bold">
            BY {vendor.toUpperCase()}
          </p>
        </div>

        {/* Rating */}
        <div className="flex items-center gap-2 mb-4">
          <div className="flex">
            {[...Array(5)].map((_, i) => (
              <Star
                key={i}
                className={`w-4 h-4 ${
                  i < rating ? "fill-primary text-primary" : "text-muted"
                }`}
              />
            ))}
          </div>
          <span className="text-sm font-bold text-muted-foreground">
            {rating}/5
          </span>
        </div>

        {/* Price and Action */}
        <div className="flex items-center justify-between">
          <div>
            <div className="text-2xl font-black text-primary">{price}</div>
            <div className="text-xs font-bold text-muted-foreground">BTC</div>
          </div>
          <Button variant="default" size="sm" className="font-bold">
            VIEW
          </Button>
        </div>
      </div>
    </Card>
  );
};

export default ProductCard;
