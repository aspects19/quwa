import { logout } from "@/lib/appwrite";
import { LogOut } from "lucide-react";

export default function Header() {
  return (
    <header className="w-full flex flex-row justify-between px-6 mt-6 mb-2 py-2 sticky top-0 z-50">
      <h1 className="text-2xl font-bold gradient-text ">
        Quwa
      </h1>
      <p className="text-sm text-dark-text-muted">
        powered by Gemini
      </p>
      <button className="btn btn-outline btn-accent" onClick={async () => await logout()}>
        <LogOut size={20} />
      </button>
    </header>
  )
}
