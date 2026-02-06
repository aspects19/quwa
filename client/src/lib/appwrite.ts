import { Client, Account, ID } from 'appwrite';
import { jwtDecode } from 'jwt-decode';

export const config = {
  endpoint: import.meta.env.VITE_APPWRITE_ENDPOINT,
  projectId: import.meta.env.VITE_APPWRITE_PROJECT_ID,
};

const client: Client = new Client();

client.setEndpoint(config.endpoint).setProject(config.projectId);

export const account: Account = new Account(client);

interface JWTPayload {
  exp: number;
  [key: string]: any;
}

function isTokenExpiringSoon(token: string): boolean {
  try {
    const decoded = jwtDecode<JWTPayload>(token);
    const currentTime = Math.floor(Date.now() / 1000);
    const expirationBuffer = 60; 
    
    return decoded.exp < (currentTime + expirationBuffer);
  } catch (error) {
    return true;
  }
}

export async function signup(email: string, password: string, name: string) {
  try {
    const newAccount = await account.create({
      userId: ID.unique(),
      email,
      password,
      name
    });
    
    if (!newAccount) throw new Error('Error creating account');

    await login(email, password);
    
    return newAccount;
  } catch (error) {
    console.error('Error creating account:', error);
    throw error;
  }
}

export async function login(email: string, password: string) {
  try {
    const session = await account.createEmailPasswordSession({ email, password });
    const jwt = await account.createJWT();
    localStorage.setItem('jwt_token', jwt.jwt);
    
    return session;
  } catch (error) {
    console.error('Error logging in:', error);
    throw error;
  }
}

export async function logout() {
  try {
    await account.deleteSession({ sessionId: 'current' });
    localStorage.removeItem('jwt_token');
  } catch (error) {
    console.error('Error logging out:', error);
    throw error;
  }
}

export async function getCurrentUser() {
  try {
    return await account.get();
  } catch (error) {
    return null;
  }
}

export async function getValidJWT(): Promise<string> {
  try {
    const cachedToken = localStorage.getItem('jwt_token');
    
    if (cachedToken && !isTokenExpiringSoon(cachedToken)) {
      return cachedToken;
    }
    
    const jwt = await account.createJWT();
    localStorage.setItem('jwt_token', jwt.jwt);
    return jwt.jwt;
  } catch (error) {
    console.error('Error creating JWT:', error);
    throw new Error('Failed to get authentication token. Please log in again.');
  }
}



