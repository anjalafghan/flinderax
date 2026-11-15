import React, { useCallback, useEffect, useState } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { clearSession, isCookieSession } from '../session';
import { AuthContext } from './AuthContext';

export const AuthProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
    // A token whose expiry has passed is dropped while the state is first made, so the page never shows it as signed in.
    const [token, setToken] = useState<string | null>(() => {
        const storedToken = localStorage.getItem('token');
        const expiresAt = localStorage.getItem('expires_at');
        // A cookie login is renewed in its cookie, so only the server knows when it really ends (a 401).
        if (storedToken && !isCookieSession(storedToken) && expiresAt && Date.now() > parseInt(expiresAt) * 1000) {
            clearSession();
            return null;
        }
        return storedToken;
    });
    // The token is read synchronously above, so there is nothing to wait for.
    const isLoading = false;
    const queryClient = useQueryClient();

    const logout = useCallback(() => {
        // Ends the cookie on the server (best effort: a stale cookie is harmless and expires on its own).
        if (isCookieSession(localStorage.getItem('token'))) {
            try {
                void fetch('/api/common/logout', { method: 'POST', headers: { 'Content-Type': 'application/json', 'X-Joy': '1' }, body: '{}' }).catch(() => {});
            } catch {
                /* ignore */
            }
        }
        // The token, drafts and saved offline copies all belong to this login: drop them with it.
        clearSession();
        // Cached responses belong to this login too; the next user must not see them.
        queryClient.clear();
        setToken(null);
    }, [queryClient]);

    useEffect(() => {
        const expiresAt = localStorage.getItem('expires_at');
        if (token && !isCookieSession(token) && expiresAt && Date.now() > parseInt(expiresAt) * 1000) {
            // The page is leaving, so clear storage directly: the reload resets the state.
            clearSession();
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
