import { Dices, Map, Network, Swords, Trophy } from "lucide-react";
import SectionCard from "../components/SectionCard";

export default function Rules() {
    return (
        <div className="max-w-3xl mx-auto px-6 py-16">
            <h1 className="text-4xl font-bold mb-2">Game Rules</h1>
            <p className="text-ctp-subtext0 mb-12">Everything you need to know to play rsdice.</p>

            <div className="space-y-6">
                <SectionCard icon={<Map className="w-5 h-5" />} title="Setup">
                    <ul className="list-disc list-inside space-y-2 text-ctp-subtext1">
                        <li>The map is built from hexagonal tiles grouped into <strong className="text-ctp-text">areas</strong>.</li>
                        <li>Up to <strong className="text-ctp-text">6 players</strong> can join a game.</li>
                        <li>Each player is randomly assigned a set of areas at the start.</li>
                        <li>Every owned area begins with a <strong className="text-ctp-text">random number of dice (1–8)</strong>.</li>
                    </ul>
                </SectionCard>

                <SectionCard icon={<Swords className="w-5 h-5" />} title="Taking a Turn">
                    <ul className="list-disc list-inside space-y-2 text-ctp-subtext1">
                        <li>Players take turns in order.</li>
                        <li>On your turn you may attack any <strong className="text-ctp-text">adjacent enemy area</strong> from one of your own areas.</li>
                        <li>You can make as many attacks as you like before ending your turn.</li>
                    </ul>
                </SectionCard>

                <SectionCard icon={<Dices className="w-5 h-5" />} title="Combat">
                    <ul className="list-disc list-inside space-y-2 text-ctp-subtext1">
                        <li>The <strong className="text-ctp-text">attacker</strong> rolls the dice on their attacking area; the <strong className="text-ctp-text">defender</strong> rolls the dice on their defending area.</li>
                        <li>
                            If the attacker's total is <strong className="text-ctp-text">strictly greater</strong>, the attacker captures the area — all dice except one move from the attacking area to the captured area.
                        </li>
                        <li>
                            On a <strong className="text-ctp-text">draw or defender higher</strong>, the attacker loses all dice on the attacking area except one.
                        </li>
                    </ul>
                </SectionCard>

                <SectionCard icon={<Network className="w-5 h-5" />} title="End of Turn &amp; Bonus Dice">
                    <ul className="list-disc list-inside space-y-2 text-ctp-subtext1">
                        <li>At the end of your turn you receive <strong className="text-ctp-text">bonus dice equal to the size of your largest group of connected areas</strong>.</li>
                        <li>Bonus dice are distributed randomly across all your areas. Each area is capped at <strong className="text-ctp-text">8 dice</strong>.</li>
                        <li>Excess dice that cannot be placed are <strong className="text-ctp-text">stored</strong> (up to 20) and carried over to future turns.</li>
                    </ul>
                </SectionCard>

                <SectionCard icon={<Trophy className="w-5 h-5" />} title="Winning">
                    <ul className="list-disc list-inside space-y-2 text-ctp-subtext1">
                        <li>A player is <strong className="text-ctp-text">eliminated</strong> when they lose all their areas.</li>
                        <li>The <strong className="text-ctp-text">last player standing</strong> wins the game.</li>
                    </ul>
                </SectionCard>
            </div>
        </div>
    );
}
