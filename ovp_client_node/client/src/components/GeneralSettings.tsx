// ./components/GeneralSettings.tsx
import React from "react";
const GeneralSettings: React.FC = () => {
    return (
        <div className="bg-gray-700 p-4 rounded-lg shadow-md">
            <h2 className="text-xl font-semibold mb-2">General Settings</h2>
            <form>
                <div className="mb-4">
                    <label htmlFor="username" className="block text-sm font-medium text-gray-300">
                        Username
                    </label>
                    <input
                        type="text"
                        id="username"
                        className="mt-1 p-2 w-full bg-gray-800 text-gray-300 border border-gray-600 rounded-md focus:outline-none focus:border-blue-500"
                    />
                </div>
                <div className="mb-4">
                    <label htmlFor="email" className="block text-sm font-medium text-gray-300">
                        Email
                    </label>
                    <input
                        type="email"
                        id="email"
                        className="mt-1 p-2 w-full bg-gray-800 text-gray-300 border border-gray-600 rounded-md focus:outline-none focus:border-blue-500"
                    />
                </div>
                <div className="mb-4">
                    <label htmlFor="password" className="block text-sm font-medium text-gray-300">
                        Password
                    </label>
                    <input
                        type="password"
                        id="password"
                        className="mt-1 p-2 w-full bg-gray-800 text-gray-300 border border-gray-600 rounded-md focus:outline-none focus:border-blue-500"
                    />
                </div>
            </form>
        </div>
    );
};

export default GeneralSettings;