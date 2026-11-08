import { createContext, useContext } from 'react';


export interface AuthContextType {
    isAuthenticated: boolean;
    token: string | null;
    login: (token: string, expiresAt: number) => void;
    logout: () => void;
    isLoading: boolean;
}

export const AuthContext = createContext<AuthContextType | undefined>(undefined);

export const useAuth = () => {
    const context = useContext(AuthContext);
    if (context === undefined) {
        throw new Error('useAuth must be used within an AuthProvider');
    }
    return context;
};
