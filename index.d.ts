/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

export interface ReplacedLink {
  href: string
  tracked: string
}
export interface ReplacedUser {
  name?: string
  surname?: string
}
export declare function replaceHandlebarsTokens(buffer: Buffer, data: Record<string, unknown>): Buffer
export declare function findAllHrefs(buffer: Buffer, excluded?: Array<string> | undefined | null): Array<string>
export declare function findHandlebarsTokens(buffer: Buffer): Array<string>
export declare function addPreHeader(buffer: Buffer, header: string): Buffer
export declare function addPreHeaderAndLinks(buffer: Buffer, links: Array<ReplacedLink>, openLink: string, header?: string | undefined | null, userId?: string | undefined | null): Buffer
