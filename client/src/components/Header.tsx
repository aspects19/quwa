import { User } from "lucide-react";

export default function Header() {
  return (
    <header className="w-full flex flex-row justify-between items-center px-6 mt-6 mb-2 py-4 sticky top-0 z-50">
      <h1 className="text-2xl font-bold gradient-text hover-lift cursor-pointer">
        Quwa
      </h1>
      <p className="text-sm text-white/60 hover:text-white/80 transition-colors">
        powered by Gemini
      </p>
      <button className="w-12 h-12 rounded-full bg-white/5 hover:bg-white/10 items-center justify-center flex border border-white/20 hover:border-white/40 hover-lift transition-all group">
        <User size={25} className="text-white/70 group-hover:text-white transition-colors" />
      </button>
    </header>
  )
}
