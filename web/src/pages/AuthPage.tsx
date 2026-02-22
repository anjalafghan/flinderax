import { useMutation } from "@tanstack/react-query"
import { useNavigate } from "react-router-dom"
import { Wallet } from "lucide-react"
import { useState } from "react"
import { toast } from "sonner"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { ModeToggle } from "@/components/ui/mode-toggle"
import api from "@/services/api"
import { useAuth } from "@/context/AuthContext"

export default function AuthPage() {
    const navigate = useNavigate()
    const { login } = useAuth()

    // Login State
    const [loginUser, setLoginUser] = useState("")
    const [loginPass, setLoginPass] = useState("")

    // Register State
    const [regUser, setRegUser] = useState("")
    const [regPass, setRegPass] = useState("")

    const loginMutation = useMutation({
        mutationFn: async () => {
            const res = await api.post<{ access_token: string, expires_at: number }>("/common/login", {
                user_name: loginUser,
                user_password: loginPass,
            })
            return res.data
        },
        onSuccess: (data) => {
            toast.success("Welcome back!")
            login(data.access_token, data.expires_at)
            navigate("/")
        },
        onError: (error: any) => {
            toast.error(error.response?.data || "Login failed. Please check your credentials.")
        },
    })

    const registerMutation = useMutation({
        mutationFn: async () => {
            // Backend requires user_role
            const res = await api.post<any>("/common/register", {
                user_name: regUser,
                user_password: regPass,
                user_role: "user",
            })
            return res.data
        },
        onSuccess: () => {
            toast.success("Account created successfully! Please login.")
            // Switch to login tab? For now just notify.
            // Reset fields
            setRegUser("")
            setRegPass("")
        },
        onError: (error: any) => {
            toast.error(error.response?.data || "Registration failed. Please try again.")
        },
    })

    const handleLogin = (e: React.FormEvent) => {
        e.preventDefault()
        if (!loginUser || !loginPass) {
            toast.error("Please fill in all fields")
            return
        }
        loginMutation.mutate()
    }

    const handleRegister = (e: React.FormEvent) => {
        e.preventDefault()
        if (!regUser || !regPass) {
            toast.error("Please fill in all fields")
            return
        }
        registerMutation.mutate()
    }

    return (
        <div className="relative flex min-h-screen items-center justify-center bg-background p-4 text-foreground">
            <div className="absolute top-4 right-4">
                <ModeToggle />
            </div>
            <div className="w-full max-w-md">
                <div className="mb-8 text-center space-y-4">
                    <div className="mx-auto flex h-16 w-16 items-center justify-center rounded-2xl bg-gradient-to-br from-gray-900 to-black dark:from-gray-100 dark:to-white text-white dark:text-gray-900 shadow-xl rotate-3">
                        <Wallet className="h-8 w-8" />
                    </div>
                    <div className="space-y-1">
                        <h1 className="text-xl font-bold tracking-tight text-slate-900 dark:text-white opacity-80 uppercase tracking-[0.2em]">Flinderax</h1>
                        <h2 className="text-4xl md:text-5xl font-black tracking-tighter text-slate-900 dark:text-white leading-none">Your real balance.</h2>
                        <p className="text-muted-foreground font-medium text-lg leading-snug max-w-sm mx-auto pt-2">
                            Separate what you’ve spent from what you can spend.
                        </p>
                    </div>
                </div>

                <div className="bg-card p-2 rounded-2xl shadow-lg border border-border dark:shadow-2xl">
                    <Tabs defaultValue="login" className="w-full">
                        <TabsList className="grid w-full grid-cols-2 mb-2 p-1 bg-muted/30 dark:bg-muted/50 rounded-xl">
                            <TabsTrigger value="login" className="rounded-lg data-[state=active]:bg-background data-[state=active]:shadow-sm">Login</TabsTrigger>
                            <TabsTrigger value="register" className="rounded-lg data-[state=active]:bg-background data-[state=active]:shadow-sm">Register</TabsTrigger>
                        </TabsList>

                        <TabsContent value="login" className="mt-0">
                            <Card className="border-0 shadow-none">
                                <CardHeader>
                                    <CardTitle>Welcome Back</CardTitle>
                                    <CardDescription>
                                        Enter your credentials to access your wallet.
                                    </CardDescription>
                                </CardHeader>
                                <form onSubmit={handleLogin}>
                                    <CardContent className="space-y-4">
                                        <div className="space-y-2">
                                            <Label htmlFor="username">Username</Label>
                                            <Input
                                                id="username"
                                                placeholder="e.g. johndoe"
                                                className="bg-muted/50 border-border focus-visible:ring-ring h-11"
                                                value={loginUser}
                                                onChange={(e) => setLoginUser(e.target.value)}
                                            />
                                        </div>
                                        <div className="space-y-2">
                                            <Label htmlFor="password">Password</Label>
                                            <Input
                                                id="password"
                                                type="password"
                                                placeholder="••••••••"
                                                className="bg-muted/50 border-border focus-visible:ring-ring h-11"
                                                value={loginPass}
                                                onChange={(e) => setLoginPass(e.target.value)}
                                            />
                                        </div>
                                    </CardContent>
                                    <CardFooter>
                                        <Button type="submit" className="w-full h-11 bg-primary hover:bg-primary/90 text-primary-foreground font-medium text-base rounded-xl" disabled={loginMutation.isPending}>
                                            {loginMutation.isPending ? "Logging in..." : "Login"}
                                        </Button>
                                    </CardFooter>
                                </form>
                            </Card>
                        </TabsContent>

                        <TabsContent value="register" className="mt-0">
                            <Card className="border-0 shadow-none">
                                <CardHeader>
                                    <CardTitle>Create Account</CardTitle>
                                    <CardDescription>
                                        Start your financial journey today.
                                    </CardDescription>
                                </CardHeader>
                                <form onSubmit={handleRegister}>
                                    <CardContent className="space-y-4">
                                        <div className="space-y-2">
                                            <Label htmlFor="reg-username">Username</Label>
                                            <Input
                                                id="reg-username"
                                                placeholder="Choose a username"
                                                className="bg-muted/50 border-border focus-visible:ring-ring h-11"
                                                value={regUser}
                                                onChange={(e) => setRegUser(e.target.value)}
                                            />
                                        </div>
                                        <div className="space-y-2">
                                            <Label htmlFor="reg-password">Password</Label>
                                            <Input
                                                id="reg-password"
                                                type="password"
                                                placeholder="Create a password"
                                                className="bg-muted/50 border-border focus-visible:ring-ring h-11"
                                                value={regPass}
                                                onChange={(e) => setRegPass(e.target.value)}
                                            />
                                        </div>
                                    </CardContent>
                                    <CardFooter>
                                        <Button type="submit" className="w-full h-11 bg-primary hover:bg-primary/90 text-primary-foreground font-medium text-base rounded-xl" disabled={registerMutation.isPending}>
                                            {registerMutation.isPending ? "Creating..." : "Register"}
                                        </Button>
                                    </CardFooter>
                                </form>
                            </Card>
                        </TabsContent>
                    </Tabs>
                </div>
            </div>
        </div>
    )
}
