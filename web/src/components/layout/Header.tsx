import { useNavigate } from "react-router-dom"
import { Plus, LogOut } from "lucide-react"
import { Button } from "@/components/ui/button"
import { ModeToggle } from "@/components/ui/mode-toggle"
import { useAuth } from "@/context/AuthContext"

export function Header() {
    const navigate = useNavigate()
    const { logout } = useAuth()

    const handleLogout = () => {
        logout()
        navigate('/auth')
    }

    return (
        <header className="sticky top-0 z-50 w-full border-b bg-background/80 backdrop-blur-md dark:bg-background/80 dark:border-white/10">
            <div className="container mx-auto flex h-16 items-center justify-between px-6">
                <div className="flex items-center gap-2 cursor-pointer" onClick={() => navigate('/')}>
                    <div className="h-8 w-8 rounded-lg bg-primary text-primary-foreground flex items-center justify-center font-bold">F</div>
                    <span className="text-xl font-bold tracking-tight text-slate-900 dark:text-white">Flinderax</span>
                </div>

                <div className="flex items-center gap-2 md:gap-4">
                    <Button variant="outline" size="sm" onClick={() => navigate('/cards/new')} className="hidden md:flex">
                        <Plus className="mr-2 h-4 w-4" /> Add Card
                    </Button>

                    <ModeToggle />

                    <Button variant="ghost" size="icon" onClick={handleLogout} title="Logout">
                        <LogOut className="h-[1.2rem] w-[1.2rem]" />
                    </Button>
                </div>
            </div>
        </header>
    )
}
