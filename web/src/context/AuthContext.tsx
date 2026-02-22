import React, { createContext, useContext, useState, useEffect } from 'react';


interface AuthContextType {
    isAuthenticated: boolean;
    token: string | null;
    login: (token: string, expiresAt: number) => void;
    logout: () => void;
    isLoading: boolean;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export const AuthProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
    const [token, setToken] = useState<string | null>(localStorage.getItem('token'));
    const [isLoading, setIsLoading] = useState(true);

    useEffect(() => {
        // Check if token exists on mount
        const storedToken = localStorage.getItem('token');
        const expiresAt = localStorage.getItem('expires_at');

        if (storedToken && expiresAt) {
            if (Date.now() > parseInt(expiresAt) * 1000) {
                localStorage.removeItem('token');
                localStorage.removeItem('expires_at');
                setToken(null);
            } else {
                setToken(storedToken);
            }
        } else if (storedToken && !expiresAt) {
            // Legacy token handling or just keep it
            setToken(storedToken);
        }
        setIsLoading(false);
    }, []);

    useEffect(() => {
        const expiresAt = localStorage.getItem('expires_at');
        if (expiresAt && Date.now() > parseInt(expiresAt) * 1000) {
            logout();
            window.location.href = '/auth';
        }
    }, [token]);

    const login = (newToken: string, expiresAt: number) => {
        localStorage.setItem('token', newToken);
        localStorage.setItem('expires_at', expiresAt.toString());
        setToken(newToken);
    };

    const logout = () => {
        localStorage.removeItem('token');
        localStorage.removeItem('expires_at');
        setToken(null);
    };

    return (
        <AuthContext.Provider value={{ isAuthenticated: !!token, token, login, logout, isLoading }}>
            {children}
        </AuthContext.Provider>
    );
};

export const useAuth = () => {
    const context = useContext(AuthContext);
    if (context === undefined) {
        throw new Error('useAuth must be used within an AuthProvider');
    }
    return context;
};
