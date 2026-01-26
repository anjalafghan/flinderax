import { useState } from "react"
import { useMutation } from "@tanstack/react-query"
import { useNavigate } from "react-router-dom"
import { toast } from "sonner"
import { Wallet } from "lucide-react"

import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
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
            const res = await api.post("/common/login", {
                user_name: loginUser,
                user_password: loginPass,
            })
            return res.data
        },
        onSuccess: (data) => {
            toast.success("Welcome back!")
            login(data.access_token)
            navigate("/")
        },
        onError: (error: any) => {
            toast.error(error.response?.data || "Login failed. Please check your credentials.")
        },
    })

    const registerMutation = useMutation({
        mutationFn: async () => {
            // Backend requires user_role
            const res = await api.post("/common/register", {
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
        <div className="flex min-h-screen items-center justify-center bg-[#F8F9FB] p-4 text-gray-900">
            <div className="w-full max-w-md">
                <div className="mb-8 text-center space-y-4">
                    <div className="mx-auto flex h-16 w-16 items-center justify-center rounded-2xl bg-black text-white shadow-xl rotate-3">
                        <Wallet className="h-8 w-8" />
                    </div>
                    <div>
                        <h1 className="text-3xl font-extrabold tracking-tight">Flinderax</h1>
                        <p className="text-gray-500 font-medium">Control your wealth.</p>
                    </div>
                </div>

                <div className="bg-white p-2 rounded-2xl shadow-sm border border-gray-100">
                    <Tabs defaultValue="login" className="w-full">
                        <TabsList className="grid w-full grid-cols-2 mb-2 p-1 bg-gray-50/50 rounded-xl">
                            <TabsTrigger value="login" className="rounded-lg data-[state=active]:bg-white data-[state=active]:shadow-sm">Login</TabsTrigger>
                            <TabsTrigger value="register" className="rounded-lg data-[state=active]:bg-white data-[state=active]:shadow-sm">Register</TabsTrigger>
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
                                                className="bg-gray-50 border-gray-200 focus-visible:ring-black h-11"
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
                                                className="bg-gray-50 border-gray-200 focus-visible:ring-black h-11"
                                                value={loginPass}
                                                onChange={(e) => setLoginPass(e.target.value)}
                                            />
                                        </div>
                                    </CardContent>
                                    <CardFooter>
                                        <Button type="submit" className="w-full h-11 bg-black hover:bg-gray-800 text-white font-medium text-base rounded-xl" disabled={loginMutation.isPending}>
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
                                                className="bg-gray-50 border-gray-200 focus-visible:ring-black h-11"
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
                                                className="bg-gray-50 border-gray-200 focus-visible:ring-black h-11"
                                                value={regPass}
                                                onChange={(e) => setRegPass(e.target.value)}
                                            />
                                        </div>
                                    </CardContent>
                                    <CardFooter>
                                        <Button type="submit" className="w-full h-11 bg-black hover:bg-gray-800 text-white font-medium text-base rounded-xl" disabled={registerMutation.isPending}>
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
