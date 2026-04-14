import { Suspense } from "react";
import { NuqsAdapter } from "nuqs/adapters/react";
import { Providers } from "@/components/providers";
import { TooltipProvider } from "@/components/ui/tooltip";
import Home from "@/pages/Home";

export function App() {
	return (
		<NuqsAdapter>
			<Providers>
				<TooltipProvider>
					<Suspense>
						<Home />
					</Suspense>
				</TooltipProvider>
			</Providers>
		</NuqsAdapter>
	);
}
