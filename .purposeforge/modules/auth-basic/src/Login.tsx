import { useState } from 'react';
import { useAuth } from '../../context/AuthContext';

export default function Login() {
    const [name, setName] = useState('');
    const { login } = useAuth();

    return (
        <div className="login-box">
            <h2>Login</h2>
            <input
                type="text"
                value={name}
                onChange={e => setName(e.target.value)}
                placeholder="Enter your name"
            />
            <button onClick={() => login(name)}>Log In</button>
        </div>
    );
}
