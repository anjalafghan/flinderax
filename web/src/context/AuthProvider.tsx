import React, { useCallback, useEffect, useState } from 'react';
import { AuthContext } from './AuthContext';

export const AuthProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
    // A token whose expiry has passed is dropped while the state is first made, so the page never shows it as signed in.
    const [token, setToken] = useState<string | null>(() => {
        const storedToken = localStorage.getItem('token');
        const expiresAt = localStorage.getItem('expires_at');
        if (storedToken && expiresAt && Date.now() > parseInt(expiresAt) * 1000) {
            localStorage.removeItem('token');
            localStorage.removeItem('expires_at');
            return null;
        }
        return storedToken;
    });
    // The token is read synchronously above, so there is nothing to wait for.
    const isLoading = false;

    const logout = useCallback(() => {
        // Saved offline copies belong to this login: drop them with it.
        navigator.serviceWorker?.controller?.postMessage({ type: "logout" });
        localStorage.removeItem('token');
        localStorage.removeItem('expires_at');
        setToken(null);
    }, []);

    useEffect(() => {
        const expiresAt = localStorage.getItem('expires_at');
        if (expiresAt && Date.now() > parseInt(expiresAt) * 1000) {
            // The page is leaving, so clear storage directly: the reload resets the state.
            navigator.serviceWorker?.controller?.postMessage({ type: "logout" });
            localStorage.removeItem('token');
            localStorage.removeItem('expires_at');
            window.location.href = '/auth';
        }
    }, [token]);

    const login = (newToken: string, expiresAt: number) => {
        localStorage.setItem('token', newToken);
        localStorage.setItem('expires_at', expiresAt.toString());
        setToken(newToken);
    };

    return (
        <AuthContext.Provider value={{ isAuthenticated: !!token, token, login, logout, isLoading }}>
            {children}
        </AuthContext.Provider>
    );
};
