import ky from "ky";
import type { z } from "zod";
import { logger } from "./logger";

const log = logger.child({ module: "api" });

function safeUrl(request: Request): string {
	try {
		const u = new URL(request.url);
		return `${u.pathname}${u.search}`;
	} catch {
		return request.url;
	}
}

export const api = ky.create({
	prefixUrl: import.meta.env.VITE_API_URL ?? "http://localhost:8000",
	timeout: 10_000,
	retry: { limit: 1, statusCodes: [408, 429, 500, 502, 503, 504] },
	hooks: {
		beforeRequest: [
			(request) => {
				log.debug(
					{ method: request.method, url: safeUrl(request) },
					"api request",
				);
			},
		],
		afterResponse: [
			(request, _options, response) => {
				const level = response.ok ? "debug" : "warn";
				log[level](
					{
						method: request.method,
						url: safeUrl(request),
						status: response.status,
					},
					"api response",
				);
				return response;
			},
		],
		beforeError: [
			(error) => {
				log.error(
					{
						method: error.request.method,
						url: safeUrl(error.request),
						status: error.response?.status,
						err: error,
					},
					"api error",
				);
				return error;
			},
		],
	},
});

export interface BBox {
	south: number;
	west: number;
	north: number;
	east: number;
}

export async function typedGet<T>(
	schema: z.ZodType<T>,
	path: string,
	params?: Record<string, string>,
	signal?: AbortSignal,
): Promise<T> {
	const searchParams = params ? new URLSearchParams(params) : undefined;
	const data: unknown = await api
		.get(path, { searchParams, signal: signal ?? null })
		.json();

	const result = schema.safeParse(data);
	if (!result.success) {
		log.error(
			{ errors: result.error.flatten(), path },
			"schema validation failed",
		);
		throw new Error(`Response validation failed: ${path}`);
	}
	return result.data;
}
