import { LucideIcon } from "lucide-react";
import { Card } from "@/components/ui/card";

interface CategoryCardProps {
  icon: LucideIcon;
  title: string;
  count: number;
}

const CategoryCard = ({ icon: Icon, title, count }: CategoryCardProps) => {
  return (
    <Card className="group relative overflow-hidden bg-card hover:bg-primary transition-all duration-300 cursor-pointer border-2 border-border hover:border-primary">
      <div className="p-8">
        <div className="flex items-start justify-between mb-6">
          <Icon className="w-12 h-12 text-primary group-hover:text-primary-foreground transition-colors" />
          <div className="text-right">
            <div className="text-3xl font-black text-primary group-hover:text-primary-foreground transition-colors">
              {count}
            </div>
            <div className="text-xs font-bold text-muted-foreground group-hover:text-primary-foreground/60 transition-colors">
              ITEMS
            </div>
          </div>
        </div>
        <h3 className="text-2xl font-black tracking-tight group-hover:text-primary-foreground transition-colors">
          {title}
        </h3>
      </div>
      
      {/* Hover Effect */}
      <div className="absolute inset-0 bg-primary opacity-0 group-hover:opacity-10 transition-opacity" />
    </Card>
  );
};

export default CategoryCard;
