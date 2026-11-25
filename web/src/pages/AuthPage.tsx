import { loginToKeep } from "@/session";
import { useMutation } from "@tanstack/react-query";
import { useNavigate } from "react-router-dom";
import { lazy, Suspense, useState } from "react";
import { Eye, EyeOff, Heart, Wallet } from "lucide-react";
import { toast } from "sonner";

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
import { useSection } from "@/theme/useSection";
import GradientText from "@/components/bits/GradientText";
import ShinyText from "@/components/bits/ShinyText";
import SpotlightCard from "@/components/bits/SpotlightCard";
import StarBorder from "@/components/bits/StarBorder";

// The front door gets one WebGL hero, in its own chunk so the form paints first.
const Grainient = lazy(() => import("@/components/bits/Grainient"));

/** No hero on phones that ask for less data, or when WebGL is missing. */
function wantsHero(): boolean {
  if (typeof window === "undefined") return false;
  const nav = navigator as Navigator & { connection?: { saveData?: boolean } };
  if (nav.connection?.saveData) return false;
  try {
    return !!document.createElement("canvas").getContext("webgl2");
  } catch {
    return false;
  }
}

// Warm rainbow: marigold, coral, rose, grape, back to sky.
const HERO_LIGHT = ["#c9bfae", "#ff9a7a", "#ff7aa8", "#b79cff", "#8fd3ff"];
const HERO_DARK = ["#8a8073", "#8a8073", "#8a8073", "#8a8073", "#8a8073"];
const WORDMARK = ["#8a8073", "#8a8073", "#8a8073", "#8a8073", "#8a8073", "#8a8073"];

const TAB_TRIGGER =
  "rounded-full data-[state=active]:bg-brand-gradient data-[state=active]:text-brand-fg data-[state=active]:shadow-[0_8px_20px_-10px_hsl(var(--brand)/0.8)]";

/** The submit button: a section-gradient pill with a star running round its edge. */
function SubmitButton({
  disabled,
  children,
}: {
  disabled: boolean;
  children: React.ReactNode;
}) {
  return (
    <StarBorder
      as="button"
      type="submit"
      disabled={disabled}
      className="h-12 w-full text-base font-semibold"
      radius={24}
      thickness={1.5}
      color="#fff7d6"
      glow={0.8}
      sparkle
      duration={5}
      backgroundColor="linear-gradient(110deg, hsl(var(--brand)) 0%, hsl(var(--brand-2)) 100%)"
      textColor="hsl(var(--brand-fg))"
      borderColor="hsl(var(--brand) / 0.35)"
    >
      {children}
    </StarBorder>
  );
}

// The API rejects with an Error whose message is the server's text (see services/api.ts).
const errorMessage = (error: unknown, fallback: string) =>
  error instanceof Error && error.message ? error.message : fallback;

/** A password field with a show/hide eye, so a phone keyboard typo can be checked before submitting. */
function PasswordInput({
  shown,
  onToggle,
  ...props
}: React.ComponentProps<typeof Input> & {
  shown: boolean;
  onToggle: () => void;
}) {
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
        {shown ? (
          <EyeOff className="h-4 w-4" aria-hidden />
        ) : (
          <Eye className="h-4 w-4" aria-hidden />
        )}
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
  const [hero] = useState(wantsHero);
  const { dark } = useSection();

  const signIn = async (user: string, pass: string) => {
    const res = await api.post<{
      access_token: string;
      expires_at: number;
      cookie?: boolean;
    }>("/api/common/login", {
      user_name: user,
      user_password: pass,
    });
    return res.data;
  };
  const finishSignIn = (
    data: Awaited<ReturnType<typeof signIn>>,
    welcome: string,
  ) => {
    toast.success(welcome, { duration: 1200 });
    const kept = loginToKeep(data);
    login(kept.token, kept.expiresAt);
    navigate("/");
  };

  const loginMutation = useMutation({
    mutationFn: () => signIn(loginUser, loginPass),
    onSuccess: (data) => finishSignIn(data, "Welcome back!"),
    onError: (error) => {
      toast.error(
        errorMessage(error, "Login failed. Please check your credentials."),
      );
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
      toast.error(
        errorMessage(error, "Registration failed. Please try again."),
      );
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
    <div className="relative isolate flex min-h-screen items-center justify-center overflow-hidden p-4 text-foreground">
      {hero && (
        <div
          aria-hidden
          className="pointer-events-none absolute inset-0 -z-10 opacity-70 [mask-image:radial-gradient(120%_80%_at_50%_30%,black_35%,transparent_85%)] dark:opacity-60"
        >
          <Suspense fallback={null}>
            <Grainient
              colors={dark ? HERO_DARK : HERO_LIGHT}
              lightMode={!dark}
              type="radial"
              bands={2}
              speed={0.5}
              flow={1}
              grainAmount={0.06}
              intro
              dpr={1}
            />
          </Suspense>
        </div>
      )}
      <div className="absolute top-4 right-4">
        <ModeToggle />
      </div>
      <div className="w-full max-w-md">
        <div className="mb-10 space-y-3 text-center">
          <p className="eyebrow">a quiet ledger</p>
          <h1 className="font-display text-[88px] leading-none sm:text-[112px]">
            <GradientText
              colors={WORDMARK}
              animationSpeed={10}
              variant="flow"
              glow={0.35}
            >
              joy
            </GradientText>
          </h1>
          <h2 className="font-display text-3xl leading-tight">
            <ShinyText
              color={dark ? "#f5e9ff" : "#3b1d4a"}
              shineColor={dark ? "#c9bfae" : "#8a8073"}
              speed={3.2}
              delay={1.2}
            >
              Your money and your dates, in one place.
            </ShinyText>
          </h2>
          <ul
            aria-label="What joy holds"
            className="m-0 flex list-none justify-center gap-2 p-0 pt-1"
          >
            <li className="pill pill-mint">
              <Wallet className="h-3.5 w-3.5" aria-hidden />
              money
            </li>
            <li className="pill pill-rose">
              <Heart className="h-3.5 w-3.5" aria-hidden />
              dates
            </li>
          </ul>
        </div>

        <SpotlightCard
          theme={dark ? "dark" : "light"}
          spotlightColor={dark ? "#c9bfae" : "#8a8073"}
          intensity={dark ? 0.18 : 0.22}
          spotlightSize={320}
          ambient
          className="!rounded-[2rem] !p-2 backdrop-blur-xl"
          style={
            {
              "--spotlight-card-surface": dark
                ? "hsl(var(--card) / 0.78)"
                : "hsl(var(--card) / 0.82)",
              "--spotlight-card-border": "hsl(var(--brand) / 0.28)",
              "--spotlight-card-shadow":
                "0 24px 60px -28px hsl(var(--brand) / 0.55)",
            } as React.CSSProperties
          }
        >
          <Tabs
            defaultValue="login"
            value={tab}
            onValueChange={setTab}
            className="w-full"
          >
            <TabsList className="grid w-full grid-cols-2 mb-2 p-1 bg-brand-tint rounded-full h-12">
              <TabsTrigger value="login" className={TAB_TRIGGER}>
                Login
              </TabsTrigger>
              <TabsTrigger value="register" className={TAB_TRIGGER}>
                Register
              </TabsTrigger>
            </TabsList>

            <TabsContent value="login" className="mt-0">
              <Card className="border-0 bg-transparent shadow-none backdrop-blur-none">
                <CardHeader className="pb-4">
                  <CardTitle className="brand-text text-[24px]">
                    Welcome Back
                  </CardTitle>
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
                    <SubmitButton disabled={loginMutation.isPending}>
                      {loginMutation.isPending ? "Logging in..." : "Login"}
                    </SubmitButton>
                  </CardFooter>
                </form>
              </Card>
            </TabsContent>

            <TabsContent value="register" className="mt-0">
              <Card className="border-0 bg-transparent shadow-none backdrop-blur-none">
                <CardHeader className="pb-4">
                  <CardTitle className="brand-text text-[24px]">
                    Create Account
                  </CardTitle>
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
                    <SubmitButton disabled={registerMutation.isPending}>
                      {registerMutation.isPending
                        ? "Creating your account..."
                        : "Create account"}
                    </SubmitButton>
                  </CardFooter>
                </form>
              </Card>
            </TabsContent>
          </Tabs>
        </SpotlightCard>
      </div>
    </div>
  );
}
