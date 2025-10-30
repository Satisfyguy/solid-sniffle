import { Button } from "@/components/ui/button";
import { ArrowRight, LucideIcon } from "lucide-react";

interface CategoryCardProps {
  title: string;
  subtitle: string;
  colorClass: string;
  icon: LucideIcon;
  image?: string;
}

const CategoryCard = ({ title, subtitle, colorClass, icon: Icon, image }: CategoryCardProps) => {
  return (
    <div 
      className={`${colorClass} rounded-3xl p-8 relative overflow-hidden group cursor-pointer transition-transform hover:scale-105 hover:shadow-xl`}
      style={{ minHeight: "280px" }}
    >
      <div className="relative z-10 flex flex-col justify-between h-full">
        <div className="space-y-2">
          <p className="text-white/90 text-sm font-medium">{subtitle}</p>
          <h3 className="text-white text-3xl md:text-4xl font-bold">{title}</h3>
        </div>
        
        <Button 
          variant="secondary" 
          size="sm" 
          className="w-fit mt-6 bg-white text-primary hover:bg-white/90"
        >
          Browse
          <ArrowRight className="ml-2 h-4 w-4" />
        </Button>
      </div>
      
      {image && (
        <div className="absolute right-0 bottom-0 w-1/2 h-full opacity-80 group-hover:opacity-100 transition-opacity">
          <img src={image} alt={title} className="w-full h-full object-contain" />
        </div>
      )}
      
      <Icon className="absolute right-8 top-8 h-16 w-16 text-white/20" />
    </div>
  );
};

export default CategoryCard;
