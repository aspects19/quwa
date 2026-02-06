# JWT Token Integration with Backend API

## Overview
The chat interface now sends all messages to your backend API with JWT authentication included in the Authorization header.

## Implementation Details

### 1. Environment Configuration

Create a `.env` file in the client directory (see [`.env.example`](file:///home/aspect/code/quwa/client/.env.example)):

```bash
VITE_BACKEND_URL=http://localhost:8000
```

### 2. JWT Helper Function ([appwrite.ts](file:///home/aspect/code/quwa/client/src/lib/appwrite.ts#L67-L77))

Added `getValidJWT()` function that:
- Creates a fresh JWT token from Appwrite
- Stores it in localStorage
- Returns the token for immediate use
- Throws error if user is not authenticated

```typescript
export async function getValidJWT(): Promise<string> {
  try {
    const jwt = await account.createJWT();
    localStorage.setItem('jwt_token', jwt.jwt);
    return jwt.jwt;
  } catch (error) {
    console.error('Error creating JWT:', error);
    throw new Error('Failed to get authentication token. Please log in again.');
  }
}
```

**Benefits:**
- Automatically refreshes JWT before each API call
- Prevents expired token errors (JWTs expire after 15 minutes)
- No manual token management needed

### 3. Message Sending with JWT ([ChatInterface.tsx](file:///home/aspect/code/quwa/client/src/components/ChatInterface.tsx#L50-L67))

When a user sends a message:

1. **Get fresh JWT token**: `const jwtToken = await getValidJWT();`
2. **Send to backend**:
```typescript
const response = await fetch(`${BACKEND_URL}/api/chat`, {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${jwtToken}`,  // JWT token included here
  },
  body: JSON.stringify({
    message: messageContent,
  }),
});
```

3. **Handle response**: Parse backend response and display to user
4. **Error handling**: Show user-friendly error messages if request fails

## Expected Backend Response Format

Your backend should return JSON in this format:

```json
{
  "response": "The AI response text",
  "thinking": [
    "Analyzing symptoms...",
    "Searching database...",
    "Generating diagnosis..."
  ]
}
```

- **`response`** (required): The AI's response to display
- **`thinking`** (optional): Array of thinking steps to show with animation

## Backend JWT Verification

On your backend, verify the JWT token like this:

### Example (Node.js with Appwrite SDK)
```javascript
import { Client, Account } from 'node-appwrite';

async function verifyJWT(req, res, next) {
  const token = req.headers.authorization?.replace('Bearer ', '');
  
  if (!token) {
    return res.status(401).json({ error: 'No token provided' });
  }

  try {
    const client = new Client()
      .setEndpoint('https://fra.cloud.appwrite.io/v1')
      .setProject('your-project-id')
      .setJWT(token);

    const account = new Account(client);
    const user = await account.get();
    
    req.user = user;  // Attach user to request
    next();
  } catch (error) {
    res.status(401).json({ error: 'Invalid token' });
  }
}
```

## How It Works

1. **User logs in** → JWT created and stored
2. **User types message** → Message sent with Authorization header
3. **Backend receives**: `Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...`
4. **Backend verifies** JWT with Appwrite
5. **Backend processes** message and returns response
6. **Frontend displays** response with thinking steps

## Configuration

To change the backend URL, update `.env`:
```bash
# Development
VITE_BACKEND_URL=http://localhost:8000

# Production
VITE_BACKEND_URL=https://api.yourapp.com
```

## Testing

1. **Start backend server** on `http://localhost:8000`
2. **Sign in to the app** to get JWT token
3. **Send a message** in the chat
4. **Check browser DevTools** → Network tab to see the request with Authorization header

## Error Handling

The implementation handles these scenarios:

- ✅ User not logged in → Error message displayed
- ✅ JWT expired → Automatically refreshed before request
- ✅ Backend unreachable → User-friendly error shown
- ✅ Invalid response → Fallback message displayed
