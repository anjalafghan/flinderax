import { useMutation } from "@tanstack/react-query";
import { useNavigate } from "react-router-dom";
;
import { useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { ModeToggle } from "@/components/ui/mode-toggle";
import api from "@/services/api";
import { useAuth } from "@/context/AuthContext";

// The API rejects with an Error whose message is the server's text (see services/api.ts).
const errorMessage = (error: unknown, fallback: string) =>
  error instanceof Error && error.message ? error.message : fallback;

export default function AuthPage() {
  const navigate = useNavigate();
  const { login } = useAuth();

  // Login State
  const [loginUser, setLoginUser] = useState("");
  const [loginPass, setLoginPass] = useState("");

  // Register State
  const [regUser, setRegUser] = useState("");
  const [regPass, setRegPass] = useState("");

  const loginMutation = useMutation({
    mutationFn: async () => {
      const res = await api.post<{ access_token: string; expires_at: number }>(
        "/api/common/login",
        {
          user_name: loginUser,
          user_password: loginPass,
        },
      );
      return res.data;
    },
    onSuccess: (data) => {
      toast.success("Welcome back!");
      login(data.access_token, data.expires_at);
      navigate("/");
    },
    onError: (error) => {
      toast.error(errorMessage(error, "Login failed. Please check your credentials."));
    },
  });

  const registerMutation = useMutation({
    mutationFn: async () => {
      // Backend requires user_role
      const res = await api.post<unknown>("/api/common/register", {
        user_name: regUser,
        user_password: regPass,
        user_role: "user",
      });
      return res.data;
    },
    onSuccess: () => {
      toast.success("Account created successfully! Please login.");
      // Switch to login tab? For now just notify.
      // Reset fields
      setRegUser("");
      setRegPass("");
    },
    onError: (error) => {
      toast.error(errorMessage(error, "Registration failed. Please try again."));
    },
  });

  const handleLogin = (e: React.FormEvent) => {
    e.preventDefault();
    if (!loginUser || !loginPass) {
      toast.error("Please fill in all fields");
      return;
    }
    loginMutation.mutate();
  };

  const handleRegister = (e: React.FormEvent) => {
    e.preventDefault();
    if (!regUser || !regPass) {
      toast.error("Please fill in all fields");
      return;
    }
    registerMutation.mutate();
  };

  return (
    <div className="relative flex min-h-screen items-center justify-center p-4 text-foreground">
      <div className="absolute top-4 right-4">
        <ModeToggle />
      </div>
      <div className="w-full max-w-md">
        <div className="mb-10 space-y-3 text-center">
          <p className="eyebrow">a quiet ledger</p>
          <h1 className="font-display text-[88px] leading-none">joy</h1>
          <h2 className="font-display text-3xl leading-tight text-foreground/80">Your money and your dates, in one place.</h2>
        </div>

        <div className="glass rounded-[2rem] p-2">
          <Tabs defaultValue="login" className="w-full">
            <TabsList className="grid w-full grid-cols-2 mb-2 p-1 bg-transparent rounded-full h-12">
              <TabsTrigger
                value="login"
                className="rounded-full data-[state=active]:bg-foreground/10 data-[state=active]:text-foreground"
              >
                Login
              </TabsTrigger>
              <TabsTrigger
                value="register"
                className="rounded-full data-[state=active]:bg-foreground/10 data-[state=active]:text-foreground"
              >
                Register
              </TabsTrigger>
            </TabsList>

            <TabsContent value="login" className="mt-0">
              <Card className="border-0 bg-transparent shadow-none backdrop-blur-none">
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
                        className="h-12"
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
                        className="h-12"
                        value={loginPass}
                        onChange={(e) => setLoginPass(e.target.value)}
                      />
                    </div>
                  </CardContent>
                  <CardFooter>
                    <Button
                      type="submit"
                      className="h-12 w-full text-base"
                      disabled={loginMutation.isPending}
                    >
                      {loginMutation.isPending ? "Logging in..." : "Login"}
                    </Button>
                  </CardFooter>
                </form>
              </Card>
            </TabsContent>

            <TabsContent value="register" className="mt-0">
              <Card className="border-0 bg-transparent shadow-none backdrop-blur-none">
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
                        className="h-12"
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
                        className="h-12"
                        value={regPass}
                        onChange={(e) => setRegPass(e.target.value)}
                      />
                    </div>
                  </CardContent>
                  <CardFooter>
                    <Button
                      type="submit"
                      className="h-12 w-full text-base"
                      disabled={registerMutation.isPending}
                    >
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
  );
}
