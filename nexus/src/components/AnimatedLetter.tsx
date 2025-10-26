import { useState } from "react";
interface AnimatedLetterProps {
  letter: string;
  delay?: number;
}
const AnimatedLetter = ({
  letter,
  delay = 0
}: AnimatedLetterProps) => {
  const [isJumping, setIsJumping] = useState(false);
  const handleMouseEnter = () => {
    setIsJumping(true);
    setTimeout(() => {
      setIsJumping(false);
    }, 1500);
  };

  // Style spécial pour le E avec lignes horizontales
  if (letter === "E") {
    return <span onMouseEnter={handleMouseEnter} className={`inline-block cursor-pointer transition-all align-baseline ${isJumping ? "animate-[jump_1.5s_cubic-bezier(0.6,0,0.2,1.5)]" : ""}`} style={{
      animationDelay: `${delay}ms`
    }}>
        <span className="relative inline-block align-baseline">
          <span className="invisible">E</span>
          <span className="absolute inset-0 flex flex-col justify-around items-start py-[0.05em]">
            <span className="h-[0.15em] w-full bg-primary-foreground"></span>
            <span className="h-[0.15em] w-full bg-primary-foreground"></span>
            <span className="h-[0.15em] w-full bg-primary-foreground"></span>
          </span>
        </span>
      </span>;
  }
  return <span onMouseEnter={handleMouseEnter} className={`inline-block cursor-pointer transition-all align-baseline ${isJumping ? "animate-[jump_1.5s_cubic-bezier(0.6,0,0.2,1.5)]" : ""}`} style={{
    animationDelay: `${delay}ms`
  }}>
      {letter}
    </span>;
};
export default AnimatedLetter;