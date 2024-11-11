// ./src/components/ui/loading-spinner.tsx
import { cn } from "@/lib/utilities";

export function LoadingSpinner({
    className,
    ...props
}: React.HTMLAttributes<HTMLDivElement>) {
    return (
        <div
            className={cn("flex items-center justify-center", className)}
            {...props}
        >
            <img src="/public/assets/o.png" alt="Overpass Logo" className="op-spinner" />
        </div>
    );
}

export default LoadingSpinner;
