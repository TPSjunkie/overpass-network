// ./components/ThemeSettings.tsx
import React, { useState } from "react";
interface ThemeSettingsProps {
    onChange: (theme: "light" | "dark") => void;
    currentTheme: "light" | "dark";
}
const ThemeSettings: React.FC<ThemeSettingsProps> = ({ onChange, currentTheme }) => {
    const [theme, setTheme] = useState<"light" | "dark">(currentTheme);
    const handleThemeChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
        const selectedTheme = e.target.value as "light" | "dark";
        setTheme(selectedTheme);
        onChange(selectedTheme);
    };
    return (
        <div className="theme-settings">
            <label htmlFor="theme-select">Select Theme:</label>
            <select
                id="theme-select"
                value={theme}
                onChange={handleThemeChange}
            >
                <option value="light">Light</option>
                <option value="dark">Dark</option>
            </select>
        </div>
    );
};
export default ThemeSettings;
