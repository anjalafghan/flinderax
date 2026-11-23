import { loginToKeep } from "@/session";
import { useMutation } from "@tanstack/react-query";
import { useNavigate } from "react-router-dom";
;
import { useState } from "react";
import { Eye, EyeOff } from "lucide-react";
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

/** A password field with a show/hide eye, so a phone keyboard typo can be checked before submitting. */
function PasswordInput({
  shown,
  onToggle,
  ...props
}: React.ComponentProps<typeof Input> & { shown: boolean; onToggle: () => void }) {
  return (
    <div className="relative">
      <Input {...props} type={shown ? "text" : "password"} className="pr-12" />
      <button
        type="button"
        onClick={onToggle}
        aria-label={shown ? "Hide password" : "Show password"}
        aria-pressed={shown}
        className="absolute right-1 top-1/2 inline-flex h-11 w-11 -translate-y-1/2 items-center justify-center rounded-full text-muted-foreground hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
      >
        {shown ? <EyeOff className="h-4 w-4" aria-hidden /> : <Eye className="h-4 w-4" aria-hidden />}
      </button>
    </div>
  );
}

export default function AuthPage() {
  const navigate = useNavigate();
  const { login } = useAuth();

  // Login State
  const [loginUser, setLoginUser] = useState("");
  const [loginPass, setLoginPass] = useState("");

  // Register State
  const [regUser, setRegUser] = useState("");
  const [regPass, setRegPass] = useState("");
  const [regCode, setRegCode] = useState("");

  const [showPass, setShowPass] = useState(false);
  const [tab, setTab] = useState("login");

  const signIn = async (user: string, pass: string) => {
    const res = await api.post<{ access_token: string; expires_at: number; cookie?: boolean }>("/api/common/login", {
      user_name: user,
      user_password: pass,
    });
    return res.data;
  };
  const finishSignIn = (data: Awaited<ReturnType<typeof signIn>>, welcome: string) => {
    toast.success(welcome, { duration: 1200 });
    const kept = loginToKeep(data);
    login(kept.token, kept.expiresAt);
    navigate("/");
  };

  const loginMutation = useMutation({
    mutationFn: () => signIn(loginUser, loginPass),
    onSuccess: (data) => finishSignIn(data, "Welcome back!"),
    onError: (error) => {
      toast.error(errorMessage(error, "Login failed. Please check your credentials."));
    },
  });

  const registerMutation = useMutation({
    // Creating an account signs you straight in: no second form, no retyping the password.
    mutationFn: async () => {
      await api.post<unknown>("/api/common/register", {
        user_name: regUser,
        user_password: regPass,
        signup_code: regCode,
      });
      try {
        return await signIn(regUser, regPass);
      } catch {
        return null;
      }
    },
    onSuccess: (data) => {
      if (data) return finishSignIn(data, "Welcome to joy");
      // The account exists but the automatic sign-in failed: land on Login with the name filled in.
      toast.success("Account created. Sign in to continue.");
      setLoginUser(regUser);
      setRegPass("");
      setRegCode("");
      setTab("login");
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
          <Tabs defaultValue="login" value={tab} onValueChange={setTab} className="w-full">
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
                <CardHeader className="pb-4">
                  <CardTitle className="text-[24px]">Welcome Back</CardTitle>
                  <CardDescription>
                    Sign in to pick up where you left off.
                  </CardDescription>
                </CardHeader>
                <form onSubmit={handleLogin}>
                  <CardContent className="space-y-5 pt-0 md:pt-0">
                    <div className="field">
                      <Label htmlFor="username">Username</Label>
                      <Input
                        id="username"
                        autoComplete="username"
                        autoCapitalize="none"
                        autoCorrect="off"
                        placeholder="e.g. johndoe"
                        value={loginUser}
                        onChange={(e) => setLoginUser(e.target.value)}
                      />
                    </div>
                    <div className="field">
                      <Label htmlFor="password">Password</Label>
                      <PasswordInput
                        id="password"
                        autoComplete="current-password"
                        placeholder="••••••••"
                        value={loginPass}
                        onChange={(e) => setLoginPass(e.target.value)}
                        shown={showPass}
                        onToggle={() => setShowPass((v) => !v)}
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
                <CardHeader className="pb-4">
                  <CardTitle className="text-[24px]">Create Account</CardTitle>
                  <CardDescription>
                    Your money and your dates, ready in a minute.
                  </CardDescription>
                </CardHeader>
                <form onSubmit={handleRegister}>
                  <CardContent className="space-y-5 pt-0 md:pt-0">
                    <div className="field">
                      <Label htmlFor="reg-username">Username</Label>
                      <Input
                        id="reg-username"
                        autoComplete="username"
                        autoCapitalize="none"
                        autoCorrect="off"
                        placeholder="Choose a username"
                        value={regUser}
                        onChange={(e) => setRegUser(e.target.value)}
                      />
                    </div>
                    <div className="field">
                      <Label htmlFor="reg-code">Sign-up code</Label>
                      <Input
                        id="reg-code"
                        autoComplete="off"
                        placeholder="From the person who runs joy"
                        value={regCode}
                        onChange={(e) => setRegCode(e.target.value)}
                      />
                    </div>
                    <div className="field">
                      <Label htmlFor="reg-password">Password</Label>
                      <PasswordInput
                        id="reg-password"
                        autoComplete="new-password"
                        placeholder="Create a password"
                        value={regPass}
                        onChange={(e) => setRegPass(e.target.value)}
                        shown={showPass}
                        onToggle={() => setShowPass((v) => !v)}
                      />
                    </div>
                  </CardContent>
                  <CardFooter>
                    <Button
                      type="submit"
                      className="h-12 w-full text-base"
                      disabled={registerMutation.isPending}
                    >
                      {registerMutation.isPending ? "Creating your account..." : "Create account"}
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
